/*
 * Copyright (c) godot-rust; Bromeon and contributors.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Functions for generating engine-provided enums.
//!
//! See also models/domain/enums.rs for other enum-related methods.

use crate::models::domain::{Enum, Enumerator, EnumeratorValue, RustTy};
use crate::special_cases;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

pub fn make_enums(enums: &[Enum], cfg_attributes: &TokenStream) -> TokenStream {
    let definitions = enums.iter().map(make_enum_definition);

    quote! {
        #( #cfg_attributes #definitions )*
    }
}

/// Creates a definition for the given enum.
///
/// This will also implement all relevant traits and generate appropriate constants for each enumerator.
pub fn make_enum_definition(enum_: &Enum) -> TokenStream {
    make_enum_definition_with(enum_, true, true)
}

pub fn make_enum_definition_with(
    enum_: &Enum,
    define_enum: bool,
    define_traits: bool,
) -> TokenStream {
    assert!(
        !(enum_.is_bitfield && enum_.is_exhaustive),
        "bitfields cannot be marked exhaustive"
    );

    // Things needed for the type definition
    let derives = enum_.derives();
    let enum_doc = make_enum_doc(enum_);
    let name = &enum_.name;

    // Values
    let enumerators = enum_.enumerators.iter().map(|enumerator| {
        make_enumerator_definition(enumerator, name.to_token_stream(), !enum_.is_exhaustive)
    });

    // Various types
    let ord_type = enum_.ord_type();
    let engine_trait = enum_.engine_trait();

    let definition = if define_enum {
        // Exhaustive enums are declared as Rust enums.
        if enum_.is_exhaustive {
            quote! {
                #[repr(i32)]
                #[derive(Debug, #( #derives ),* )]
                #( #[doc = #enum_doc] )*
                ///
                /// This enum is exhaustive; you should not expect future Godot versions to add new enumerators.
                #[allow(non_camel_case_types)]
                pub enum #name {
                    #( #enumerators )*
                }
            }
        }
        //
        // Non-exhaustive enums are declared as newtype structs with associated constants.
        else {
            // Workaround because traits are defined in separate crate, but need access to field `ord`.
            let ord_vis = (!define_traits).then(|| {
                quote! { #[doc(hidden)] pub }
            });

            let debug_impl = make_enum_debug_impl(enum_, define_traits && !enum_.is_bitfield);
            quote! {
                #[repr(transparent)]
                #[derive( #( #derives ),* )]
                #( #[doc = #enum_doc] )*
                pub struct #name {
                    #ord_vis ord: #ord_type
                }

                impl #name {
                    #( #enumerators )*
                }

                #debug_impl
            }
        }
    } else {
        TokenStream::new()
    };

    let traits = define_traits.then(|| {
        // Trait implementations
        let engine_trait_impl = make_enum_engine_trait_impl(enum_);
        let index_enum_impl = make_enum_index_impl(enum_);
        let bitwise_impls = make_enum_bitwise_operators(enum_);

        quote! {
            #engine_trait_impl
            #index_enum_impl
            #bitwise_impls

            impl crate::meta::GodotConvert for #name {
                type Via = #ord_type;
            }

            impl crate::meta::ToGodot for #name {
                type ToVia<'v> = #ord_type;

                fn to_godot(&self) -> Self::ToVia<'_> {
                    <Self as #engine_trait>::ord(*self)
                }
            }

            impl crate::meta::FromGodot for #name {
                fn try_from_godot(via: Self::Via) -> std::result::Result<Self, crate::meta::error::ConvertError> {
                    <Self as #engine_trait>::try_from_ord(via)
                        .ok_or_else(|| crate::meta::error::FromGodotError::InvalidEnum.into_error(via))
                }
            }
        }
    });

    quote! {
        #definition
        #traits
    }
}

/// Creates an implementation of `IndexEnum` for the given enum.
///
/// Returns `None` if `enum_` isn't an indexable enum.
fn make_enum_index_impl(enum_: &Enum) -> Option<TokenStream> {
    let enum_max = enum_.find_index_enum_max()?;
    let name = &enum_.name;

    Some(quote! {
        impl crate::obj::IndexEnum for #name {
            const ENUMERATOR_COUNT: usize = #enum_max;
        }
    })
}

// Creates the match cases to return the enumerator name as &str.
fn make_enum_to_str_cases(enum_: &Enum) -> TokenStream {
    let enumerators = enum_.enumerators.iter().map(|enumerator| {
        let Enumerator { name, .. } = enumerator;
        let name_str = name.to_string();
        quote! {
            Self::#name => #name_str,
        }
    });

    quote! {
        #( #enumerators )*
    }
}

/// Implement `Debug` trait for the enum.
fn make_enum_debug_impl(enum_: &Enum, use_as_str: bool) -> TokenStream {
    let enum_name = &enum_.name;
    let enum_name_str = enum_name.to_string();

    // Print the ord if no matching enumerator can be found.
    let enumerator_not_found = quote! {
        f.debug_struct(#enum_name_str)
            .field("ord", &self.ord)
            .finish()?;

        return Ok(());
    };

    // Reuse `as_str` if traits are defined and not a bitfield.
    let function_body = if use_as_str {
        quote! {
            use crate::obj::EngineEnum;

            let enumerator = self.as_str();
            if enumerator.is_empty() {
                #enumerator_not_found
            }
        }
    } else {
        let enumerators = make_enum_to_str_cases(enum_);

        quote! {
            // Many enums have duplicates, thus allow unreachable.
            // In the future, we could print sth like "ONE|TWO" instead (at least for unstable Debug).
            #[allow(unreachable_patterns)]
            let enumerator = match *self {
                #enumerators
                _ => {
                    #enumerator_not_found
                }
            };
        }
    };

    quote! {
        impl std::fmt::Debug for #enum_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                #function_body
                f.write_str(enumerator)
            }
        }
    }
}

/// Creates an implementation of the engine trait for the given enum.
///
/// This will implement the trait returned by [`Enum::engine_trait`].
fn make_enum_engine_trait_impl(enum_: &Enum) -> TokenStream {
    let name = &enum_.name;
    let engine_trait = enum_.engine_trait();

    if enum_.is_bitfield {
        quote! {
            // We may want to add this in the future.
            //
            // impl #enum_name {
            //     pub const UNSET: Self = Self { ord: 0 };
            // }

            impl #engine_trait for #name {
                fn try_from_ord(ord: u64) -> Option<Self> {
                    Some(Self { ord })
                }

                fn ord(self) -> u64 {
                    self.ord
                }
            }
        }
    } else if enum_.is_exhaustive {
        let enumerators = enum_.enumerators.iter().map(|enumerator| {
            let Enumerator {
                name,
                value: EnumeratorValue::Enum(ord),
                ..
            } = enumerator
            else {
                panic!("exhaustive enum contains bitfield enumerators")
            };

            quote! {
                #ord => Some(Self::#name),
            }
        });

        let str_functions = make_enum_str_functions(enum_);

        quote! {
            impl #engine_trait for #name {
                fn try_from_ord(ord: i32) -> Option<Self> {
                    match ord {
                        #( #enumerators )*
                        _ => None,
                    }
                }

                fn ord(self) -> i32 {
                    self as i32
                }

                #str_functions
            }
        }
    } else {
        let unique_ords = enum_.unique_ords().expect("self is an enum");
        let str_functions = make_enum_str_functions(enum_);

        quote! {
            impl #engine_trait for #name {
                fn try_from_ord(ord: i32) -> Option<Self> {
                    match ord {
                        #( ord @ #unique_ords )|* => Some(Self { ord }),
                        _ => None,
                    }
                }

                fn ord(self) -> i32 {
                    self.ord
                }

                #str_functions
            }
        }
    }
}

/// Creates the `as_str` and `godot_name` implementations for the enum.
fn make_enum_str_functions(enum_: &Enum) -> TokenStream {
    let as_str_enumerators = make_enum_to_str_cases(enum_);

    // Only enumerations with different godot names are specified.
    // `as_str` is called for the rest of them.
    let godot_different_cases = {
        let enumerators = enum_
            .enumerators
            .iter()
            .filter(|enumerator| enumerator.name != enumerator.godot_name)
            .map(|enumerator| {
                let Enumerator {
                    name, godot_name, ..
                } = enumerator;
                let godot_name_str = godot_name.to_string();
                quote! {
                    Self::#name => #godot_name_str,
                }
            });

        quote! {
            #( #enumerators )*
        }
    };

    let godot_name_match = if godot_different_cases.is_empty() {
        // If empty, all the Rust names match the Godot ones.
        // Remove match statement to avoid `clippy::match_single_binding`.
        quote! {
            self.as_str()
        }
    } else {
        quote! {
            // Many enums have duplicates, thus allow unreachable.
            #[allow(unreachable_patterns)]
            match *self {
                #godot_different_cases
                _ => self.as_str(),
            }
        }
    };

    quote! {
        #[inline]
        fn as_str(&self) -> &'static str {
            // Many enums have duplicates, thus allow unreachable.
            #[allow(unreachable_patterns)]
            match *self {
                #as_str_enumerators
                _ => "",
            }
        }

        fn godot_name(&self) -> &'static str {
            #godot_name_match
        }
    }
}

/// Creates implementations for bitwise operators for the given enum.
///
/// Currently, this is just [`BitOr`](std::ops::BitOr) for bitfields but that could be expanded in the future.
fn make_enum_bitwise_operators(enum_: &Enum) -> TokenStream {
    let name = &enum_.name;

    if enum_.is_bitfield {
        // Regular bitfield.
        quote! {
            impl std::ops::BitOr for #name {
                type Output = Self;

                #[inline]
                fn bitor(self, rhs: Self) -> Self::Output {
                    Self { ord: self.ord | rhs.ord }
                }
            }
        }
    } else if let Some(mask_enum) = special_cases::as_enum_bitmaskable(enum_) {
        // Enum that has an accompanying bitfield for masking.
        let RustTy::EngineEnum { tokens: mask, .. } = mask_enum else {
            panic!("as_enum_bitmaskable() must return enum/bitfield type")
        };

        quote! {
            impl std::ops::BitOr<#mask> for #name {
                type Output = Self;

                #[inline]
                fn bitor(self, rhs: #mask) -> Self::Output {
                    Self { ord: self.ord | i32::try_from(rhs.ord).expect("masking bitfield outside integer range") }
                }
            }

            impl std::ops::BitOr<#name> for #mask {
                type Output = #name;

                #[inline]
                fn bitor(self, rhs: #name) -> Self::Output {
                    rhs | self
                }
            }

            impl std::ops::BitOrAssign<#mask> for #name {
                #[inline]
                fn bitor_assign(&mut self, rhs: #mask) {
                    *self = *self | rhs;
                }
            }
        }
    } else {
        TokenStream::new()
    }
}
/// Returns the documentation for the given enum.
///
/// Each string is one line of documentation, usually this needs to be wrapped in a `#[doc = ...]`.
fn make_enum_doc(enum_: &Enum) -> Vec<String> {
    let mut docs = Vec::new();

    if enum_.name != enum_.godot_name {
        docs.push(format!("Godot enum name: `{}`.", enum_.godot_name))
    }

    docs
}

/// Creates a definition for `enumerator` of the type `enum_type`.
///
/// If `as_constant` is true, it will be a `const` definition like:
/// ```ignore
/// pub const NAME: enum_type = ord;
/// ```
/// Otherwise, it will be a regular enum variant like:
/// ```ignore
/// NAME = ord,
/// ```
fn make_enumerator_definition(
    enumerator: &Enumerator,
    enum_type: TokenStream,
    as_constant: bool,
) -> TokenStream {
    let Enumerator {
        name,
        godot_name,
        value,
    } = enumerator;

    let docs = if &name.to_string() != godot_name {
        let doc = format!("Godot enumerator name: `{godot_name}`");

        quote! {
            #[doc(alias = #godot_name)]
            #[doc = #doc]
        }
    } else {
        TokenStream::new()
    };

    if as_constant {
        quote! {
            #docs
            pub const #name: #enum_type = #enum_type {
                ord: #value
            };
        }
    } else {
        quote! {
            #docs
            #name = #value,
        }
    }
}
