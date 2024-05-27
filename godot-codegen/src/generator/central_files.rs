/*
 * Copyright (c) godot-rust; Bromeon and contributors.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::context::Context;
use crate::conv;
use crate::generator::{enums, gdext_build_struct};
use crate::models::domain::ExtensionApi;
use crate::util::ident;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};

pub fn make_sys_central_code(api: &ExtensionApi) -> TokenStream {
    let build_config_struct = gdext_build_struct::make_gdext_build_struct(&api.godot_version);
    let (variant_type_enum, variant_type_deprecated_enumerators) =
        make_variant_type_enum(api, true);
    let [opaque_32bit, opaque_64bit] = make_opaque_types(api);

    quote! {
        #[cfg(target_pointer_width = "32")]
        pub mod types {
            #(#opaque_32bit)*
        }
        #[cfg(target_pointer_width = "64")]
        pub mod types {
            #(#opaque_64bit)*
        }

        // ----------------------------------------------------------------------------------------------------------------------------------------------

        #build_config_struct
        #variant_type_enum

        impl VariantType {
            // This will need refactoring if VariantType is changed to a real enum.
            #[doc(hidden)]
            pub fn from_sys(enumerator: crate::GDExtensionVariantType) -> Self {
                Self { ord: enumerator as i32 }
            }

            #[doc(hidden)]
            pub fn sys(self) -> crate::GDExtensionVariantType {
                self.ord as _
            }

            #variant_type_deprecated_enumerators
        }
    }
}

pub fn make_core_central_code(api: &ExtensionApi, ctx: &mut Context) -> TokenStream {
    let VariantEnums {
        variant_ty_enumerators_pascal,
        variant_ty_enumerators_shout,
        variant_ty_enumerators_rust,
        ..
    } = make_variant_enums(api, ctx);

    let (global_enum_defs, global_reexported_enum_defs) = make_global_enums(api);
    let variant_type_traits = make_variant_type_enum(api, false).0;

    // TODO impl Clone, Debug, PartialEq, PartialOrd, Hash for VariantDispatch
    // TODO could use try_to().unwrap_unchecked(), since type is already verified. Also directly overload from_variant().
    // But this requires that all the variant types support this.
    quote! {
        use crate::builtin::*;
        use crate::engine::Object;
        use crate::obj::Gd;

        // Remaining trait impls for sys::VariantType (traits only defined in godot-core).
        #variant_type_traits

        #[allow(dead_code)]
        pub enum VariantDispatch {
            Nil,
            #(
                #variant_ty_enumerators_pascal(#variant_ty_enumerators_rust),
            )*
        }

        impl VariantDispatch {
            pub fn from_variant(variant: &Variant) -> Self {
                match variant.get_type() {
                    VariantType::NIL => Self::Nil,
                    #(
                        VariantType::#variant_ty_enumerators_shout
                            => Self::#variant_ty_enumerators_pascal(variant.to::<#variant_ty_enumerators_rust>()),
                    )*

                    // Panic can be removed as soon as VariantType is a proper, non-exhaustive enum.
                    _ => panic!("Variant type not supported: {:?}", variant.get_type()),
                }
            }
        }

        impl std::fmt::Debug for VariantDispatch {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    Self::Nil => write!(f, "null"),
                    #(
                        Self::#variant_ty_enumerators_pascal(v) => write!(f, "{v:?}"),
                    )*
                }
            }
        }

        /// Global enums and constants, generated by Godot.
        pub mod global_enums {
            use crate::sys;
            #( #global_enum_defs )*
        }

       pub mod global_reexported_enums {
            use crate::sys;
            #( #global_reexported_enum_defs )*
        }
    }
}

// ----------------------------------------------------------------------------------------------------------------------------------------------
// Implementation

struct VariantEnums {
    variant_ty_enumerators_pascal: Vec<Ident>,
    variant_ty_enumerators_shout: Vec<Ident>,
    variant_ty_enumerators_rust: Vec<TokenStream>,
}

fn make_opaque_types(api: &ExtensionApi) -> [Vec<TokenStream>; 2] {
    let mut opaque_types = [Vec::new(), Vec::new()];

    for b in api.builtin_sizes.iter() {
        let index = b.config.is_64bit() as usize;
        let type_def = make_opaque_type(&b.builtin_original_name, b.size);

        opaque_types[index].push(type_def);
    }

    opaque_types
}

fn make_opaque_type(godot_original_name: &str, size: usize) -> TokenStream {
    let name = conv::to_pascal_case(godot_original_name);
    let (first, rest) = name.split_at(1);

    // Capitalize: "int" -> "Int".
    let ident = format_ident!("Opaque{}{}", first.to_ascii_uppercase(), rest);
    quote! {
        pub type #ident = crate::opaque::Opaque<#size>;
    }
}

fn make_variant_enums(api: &ExtensionApi, ctx: &mut Context) -> VariantEnums {
    // Generate builtin methods, now with info for all types available.
    // Separate vectors because that makes usage in quote! easier.
    let len = api.builtins.len();

    let mut result = VariantEnums {
        variant_ty_enumerators_pascal: Vec::with_capacity(len),
        variant_ty_enumerators_shout: Vec::with_capacity(len),
        variant_ty_enumerators_rust: Vec::with_capacity(len),
    };

    // Note: NIL is not part of this iteration, it will be added manually.
    for builtin in api.builtins.iter() {
        let original_name = builtin.godot_original_name();
        let shout_case = builtin.godot_shout_name();
        let rust_ty = conv::to_rust_type(original_name, None, ctx);
        let pascal_case = conv::to_pascal_case(original_name);

        result
            .variant_ty_enumerators_pascal
            .push(ident(&pascal_case));
        result.variant_ty_enumerators_shout.push(ident(shout_case));
        result
            .variant_ty_enumerators_rust
            .push(rust_ty.to_token_stream());
    }

    result
}

fn make_global_enums(api: &ExtensionApi) -> (Vec<TokenStream>, Vec<TokenStream>) {
    let mut global_enum_defs = vec![];
    let mut global_reexported_enum_defs = vec![];

    for enum_ in api.global_enums.iter() {
        // Skip VariantType, which is already defined in godot-ffi.
        if enum_.name == "VariantType" {
            continue;
        }

        let def = enums::make_enum_definition(enum_);

        if enum_.is_private {
            global_reexported_enum_defs.push(def);
        } else {
            global_enum_defs.push(def);
        }
    }

    (global_enum_defs, global_reexported_enum_defs)
}

fn make_variant_type_enum(api: &ExtensionApi, is_definition: bool) -> (TokenStream, TokenStream) {
    let variant_type_enum = api
        .global_enums
        .iter()
        .find(|e| e.name == "VariantType")
        .expect("missing VariantType enum in API JSON");

    let define_enum = is_definition;
    let define_traits = !is_definition;

    let enum_definition =
        enums::make_enum_definition_with(variant_type_enum, define_enum, define_traits);
    let deprecated_enumerators = enums::make_deprecated_enumerators(variant_type_enum);

    (enum_definition, deprecated_enumerators)
}
