/*
 * Copyright (c) godot-rust; Bromeon and contributors.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//! Internal registration machinery used by proc-macro APIs.

use crate::builtin::StringName;
use crate::global::PropertyUsageFlags;
use crate::meta::{ClassName, GodotConvert, GodotType, PropertyHintInfo, PropertyInfo};
use crate::obj::GodotClass;
use crate::registry::property::{Export, Var};
use crate::{classes, sys};
use godot_ffi::GodotFfi;

/// Same as [`register_var()`], but statically verifies the `Export` trait (again) and the fact that nodes can only be exported from nodes.
pub fn register_export<C: GodotClass, T: Export>(
    property_name: &str,
    getter_name: &str,
    setter_name: &str,
    hint_info: PropertyHintInfo,
    usage: PropertyUsageFlags,
) {
    // Note: if the user manually specifies `hint`, `hint_string` or `usage` keys, and thus is routed to `register_var()` instead,
    // they can bypass this validation.
    if !C::inherits::<classes::Node>() {
        if let Some(class) = T::as_node_class() {
            panic!(
                "#[export] for Gd<{t}>: nodes can only be exported in Node-derived classes, but the current class is {c}.",
                t = class,
                c = C::class_name()
            );
        }
    }

    register_var::<C, T>(property_name, getter_name, setter_name, hint_info, usage);
}

pub fn register_var<C: GodotClass, T: Var>(
    property_name: &str,
    getter_name: &str,
    setter_name: &str,
    hint_info: PropertyHintInfo,
    usage: PropertyUsageFlags,
) {
    let info = PropertyInfo {
        variant_type: <<T as GodotConvert>::Via as GodotType>::Ffi::variant_type(),
        class_name: <T as GodotConvert>::Via::class_name(),
        property_name: StringName::from(property_name),
        hint_info,
        usage,
    };

    let class_name = C::class_name();

    register_var_or_export_inner(info, class_name, getter_name, setter_name);
}

fn register_var_or_export_inner(
    info: PropertyInfo,
    class_name: ClassName,
    getter_name: &str,
    setter_name: &str,
) {
    let getter_name = StringName::from(getter_name);
    let setter_name = StringName::from(setter_name);

    let property_info_sys = info.property_sys();

    unsafe {
        sys::interface_fn!(classdb_register_extension_class_property)(
            sys::get_library(),
            class_name.string_sys(),
            std::ptr::addr_of!(property_info_sys),
            setter_name.string_sys(),
            getter_name.string_sys(),
        );
    }
}
