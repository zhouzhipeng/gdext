/*
 * Copyright (c) godot-rust; Bromeon and contributors.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::models::domain::ClassCodegenLevel;
use crate::models::json::JsonClass;
use crate::special_cases;

use proc_macro2::{Ident, Literal, TokenStream};
use quote::{format_ident, quote};

// ----------------------------------------------------------------------------------------------------------------------------------------------

/// Small utility that turns an optional vector (often encountered as JSON deserialization type) into a slice.
pub fn option_as_slice<T>(option: &Option<Vec<T>>) -> &[T] {
    option.as_ref().map_or(&[], Vec::as_slice)
}

pub fn make_imports() -> TokenStream {
    quote! {
        use godot_ffi as sys;
        use crate::builtin::*;
        use crate::meta::{ClassName, PtrcallSignatureTuple, VarcallSignatureTuple};
        use crate::classes::native::*;
        use crate::classes::Object;
        use crate::obj::{Gd, ObjectArg, AsObjectArg};
        use crate::sys::GodotFfi as _;
    }
}

#[cfg(since_api = "4.2")]
pub fn make_string_name(identifier: &str) -> TokenStream {
    let c_string = std::ffi::CString::new(identifier).expect("CString::new() failed");
    let lit = Literal::c_string(&c_string);

    quote! { StringName::from(#lit) }
}

#[cfg(before_api = "4.2")]
pub fn make_string_name(identifier: &str) -> TokenStream {
    quote! {
        StringName::from(#identifier)
    }
}
pub fn make_sname_ptr(identifier: &str) -> TokenStream {
    quote! {
        string_names.fetch(#identifier)
    }
}

pub fn get_api_level(class: &JsonClass) -> ClassCodegenLevel {
    // Work around wrong classification in https://github.com/godotengine/godot/issues/86206.
    fn override_editor(class_name: &str) -> bool {
        cfg!(before_api = "4.3")
            && matches!(
                class_name,
                "ResourceImporterOggVorbis" | "ResourceImporterMP3"
            )
    }

    if special_cases::is_class_level_server(&class.name) {
        ClassCodegenLevel::Servers
    } else if class.api_type == "editor" || override_editor(&class.name) {
        ClassCodegenLevel::Editor
    } else if class.api_type == "core" {
        ClassCodegenLevel::Scene
    } else {
        panic!(
            "class {} has unknown API type {}",
            class.name, class.api_type
        )
    }
}

pub fn ident(s: &str) -> Ident {
    format_ident!("{}", s)
}

pub fn cstr_u8_slice(string: &str) -> Literal {
    Literal::byte_string(format!("{string}\0").as_bytes())
}

// This function is duplicated in godot-macros\src\util\mod.rs
#[rustfmt::skip]
pub fn safe_ident(s: &str) -> Ident {
    // See also: https://doc.rust-lang.org/reference/keywords.html
    match s {
        // Lexer
        | "as" | "break" | "const" | "continue" | "crate" | "else" | "enum" | "extern" | "false" | "fn" | "for" | "if"
        | "impl" | "in" | "let" | "loop" | "match" | "mod" | "move" | "mut" | "pub" | "ref" | "return" | "self" | "Self"
        | "static" | "struct" | "super" | "trait" | "true" | "type" | "unsafe" | "use" | "where" | "while"

        // Lexer 2018+
        | "async" | "await" | "dyn"

        // Reserved
        | "abstract" | "become" | "box" | "do" | "final" | "macro" | "override" | "priv" | "typeof" | "unsized" | "virtual" | "yield"

        // Reserved 2018+
        | "try"
           => format_ident!("{}_", s),

         _ => ident(s)
    }
}
