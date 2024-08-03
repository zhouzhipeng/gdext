/*
 * Copyright (c) godot-rust; Bromeon and contributors.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

mod base_test;
mod class_name_test;
mod class_rename_test;
mod dynamic_call_test;
// `get_property_list` is only supported in Godot 4.3+
#[cfg(since_api = "4.3")]
mod get_property_list_test;
mod init_level_test;
mod object_arg_test;
mod object_swap_test;
mod object_test;
mod onready_test;
mod property_template_test;
mod property_test;
mod reentrant_test;
mod singleton_test;
mod virtual_methods_test;

// Need to test this in the init level method.
pub use init_level_test::initialize_init_level_test;
