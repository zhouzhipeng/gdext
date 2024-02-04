/*
 * Copyright (c) godot-rust; Bromeon and contributors.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

// ------------------------------------------------------------------------------------------------------------------------------------------
// Compatibility

// Code generated by Rust derive macros cannot cause any deprecation warnings, due to questionable "feature"
// https://github.com/rust-lang/rust/pull/58994. Fortunately, an extra layer of indirection solves most problems: we generate a declarative
// macro that itself isn't deprecated, but _its_ expansion is. Since the expansion happens in a later step, the warning is emitted.

#[inline(always)]
#[deprecated = "#[base] is no longer needed; Base<T> is recognized directly. \n\
        More information on https://github.com/godot-rust/gdext/pull/577."]
pub const fn base_attribute() {}

#[macro_export]
macro_rules! emit_deprecated_warning {
    ($warning_fn:ident) => {
        const _: () = $crate::__deprecated::$warning_fn();
    };
}

pub use crate::emit_deprecated_warning;
