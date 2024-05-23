/*
 * Copyright (c) godot-rust; Bromeon and contributors.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

#![allow(dead_code)]

#[rustfmt::skip]
#[allow(clippy::partialeq_to_none)]
pub mod gen_ffi {
    include!(concat!(env!("OUT_DIR"), "/gen_ffi.rs"));
}
