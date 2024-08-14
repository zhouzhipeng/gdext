/*
 * Copyright (c) godot-rust; Bromeon and contributors.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::builtin::{GString, StringName};
use crate::global::{PropertyHint, PropertyUsageFlags};
use crate::meta::{ArrayElement, ClassName, GodotType, PackedArrayElement};
use crate::registry::property::{Export, Var};
use crate::sys;
use godot_ffi::VariantType;

/// Describes a property in Godot.
///
/// Abstraction of the low-level `sys::GDExtensionPropertyInfo`.
///
/// Keeps the actual allocated values (the `sys` equivalent only keeps pointers, which fall out of scope).
#[derive(Debug, Clone)]
// Note: is not #[non_exhaustive], so adding fields is a breaking change. Mostly used internally at the moment though.
pub struct PropertyInfo {
    /// Which type this property has.
    ///
    /// For objects this should be set to [`VariantType::OBJECT`], and the `class_name` field to the actual name of the class.
    ///
    /// For [`Variant`][crate::builtin::Variant], this should be set to [`VariantType::NIL`].
    pub variant_type: VariantType,

    /// Which class this property is.
    ///
    /// This should be set to [`ClassName::none()`] unless the variant type is `Object`. You can use
    /// [`GodotClass::class_name()`](crate::obj::GodotClass::class_name()) to get the right name to use here.
    pub class_name: ClassName,

    /// The name of this property in Godot.
    pub property_name: StringName,

    /// Additional type information for this property, e.g. about array types or enum values. Split into `hint` and `hint_string` members.
    ///
    /// See also [`PropertyHint`] in the Godot docs.
    ///
    /// [`PropertyHint`]: https://docs.godotengine.org/en/latest/classes/class_%40globalscope.html#enum-globalscope-propertyhint
    pub hint_info: PropertyHintInfo,

    /// How this property should be used. See [`PropertyUsageFlags`] in Godot for the meaning.
    ///
    /// [`PropertyUsageFlags`]: https://docs.godotengine.org/en/latest/classes/class_%40globalscope.html#enum-globalscope-propertyusageflags
    pub usage: PropertyUsageFlags,
}

impl PropertyInfo {
    /// Create a new `PropertyInfo` representing a property named `property_name` with type `T`.
    ///
    /// This will generate property info equivalent to what a `#[var]` attribute would.
    pub fn new_var<T: Var>(property_name: &str) -> Self {
        T::Via::property_info(property_name).with_hint_info(T::var_hint())
    }

    /// Create a new `PropertyInfo` representing an exported property named `property_name` with type `T`.
    ///
    /// This will generate property info equivalent to what an `#[export]` attribute would.
    pub fn new_export<T: Export>(property_name: &str) -> Self {
        T::Via::property_info(property_name).with_hint_info(T::export_hint())
    }

    /// Change the `hint` and `hint_string` to be the given `hint_info`.
    ///
    /// See [`export_info_functions`](crate::registry::property::export_info_functions) for functions that return appropriate `PropertyHintInfo`s for
    /// various Godot annotations.
    ///
    /// # Examples
    ///
    /// Creating an `@export_range` property.
    ///
    // TODO: Make this nicer to use.
    /// ```no_run
    /// use godot::register::property::export_info_functions;
    /// use godot::meta::PropertyInfo;
    ///
    /// let property = PropertyInfo::new_export::<f64>("my_range_property")
    ///     .with_hint_info(export_info_functions::export_range(
    ///         0.0,
    ///         10.0,
    ///         Some(0.1),
    ///         false,
    ///         false,
    ///         false,
    ///         false,
    ///         false,
    ///         false,
    ///         Some("mm".to_string()),
    ///     ));
    /// ```
    pub fn with_hint_info(self, hint_info: PropertyHintInfo) -> Self {
        Self { hint_info, ..self }
    }

    /// Create a new `PropertyInfo` representing a group in Godot.
    ///
    /// See [`EditorInspector`](https://docs.godotengine.org/en/latest/classes/class_editorinspector.html#class-editorinspector) in Godot for
    /// more information.
    pub fn new_group(group_name: &str, group_prefix: &str) -> Self {
        Self {
            variant_type: VariantType::NIL,
            class_name: ClassName::none(),
            property_name: group_name.into(),
            hint_info: PropertyHintInfo {
                hint: PropertyHint::NONE,
                hint_string: group_prefix.into(),
            },
            usage: PropertyUsageFlags::GROUP,
        }
    }

    /// Create a new `PropertyInfo` representing a subgroup in Godot.
    ///
    /// See [`EditorInspector`](https://docs.godotengine.org/en/latest/classes/class_editorinspector.html#class-editorinspector) in Godot for
    /// more information.
    pub fn new_subgroup(subgroup_name: &str, subgroup_prefix: &str) -> Self {
        Self {
            variant_type: VariantType::NIL,
            class_name: ClassName::none(),
            property_name: subgroup_name.into(),
            hint_info: PropertyHintInfo {
                hint: PropertyHint::NONE,
                hint_string: subgroup_prefix.into(),
            },
            usage: PropertyUsageFlags::SUBGROUP,
        }
    }

    /// Converts to the FFI type. Keep this object allocated while using that!
    pub fn property_sys(&self) -> sys::GDExtensionPropertyInfo {
        use crate::obj::EngineBitfield as _;
        use crate::obj::EngineEnum as _;

        sys::GDExtensionPropertyInfo {
            type_: self.variant_type.sys(),
            name: sys::SysPtr::force_mut(self.property_name.string_sys()),
            class_name: sys::SysPtr::force_mut(self.class_name.string_sys()),
            hint: u32::try_from(self.hint_info.hint.ord()).expect("hint.ord()"),
            hint_string: sys::SysPtr::force_mut(self.hint_info.hint_string.string_sys()),
            usage: u32::try_from(self.usage.ord()).expect("usage.ord()"),
        }
    }

    pub fn empty_sys() -> sys::GDExtensionPropertyInfo {
        use crate::obj::EngineBitfield as _;
        use crate::obj::EngineEnum as _;

        sys::GDExtensionPropertyInfo {
            type_: VariantType::NIL.sys(),
            name: std::ptr::null_mut(),
            class_name: std::ptr::null_mut(),
            hint: PropertyHint::NONE.ord() as u32,
            hint_string: std::ptr::null_mut(),
            usage: PropertyUsageFlags::NONE.ord() as u32,
        }
    }

    /// Consumes self and turns it into a `sys::GDExtensionPropertyInfo`, should be used together with
    /// [`free_owned_property_sys`](Self::free_owned_property_sys).
    ///
    /// This will leak memory unless used together with `free_owned_property_sys`.
    pub(crate) fn into_owned_property_sys(self) -> sys::GDExtensionPropertyInfo {
        use crate::obj::EngineBitfield as _;
        use crate::obj::EngineEnum as _;

        sys::GDExtensionPropertyInfo {
            type_: self.variant_type.sys(),
            name: self.property_name.into_owned_string_sys(),
            class_name: sys::SysPtr::force_mut(self.class_name.string_sys()),
            hint: u32::try_from(self.hint_info.hint.ord()).expect("hint.ord()"),
            hint_string: self.hint_info.hint_string.into_owned_string_sys(),
            usage: u32::try_from(self.usage.ord()).expect("usage.ord()"),
        }
    }

    /// Properly frees a `sys::GDExtensionPropertyInfo` created by [`into_owned_property_sys`](Self::into_owned_property_sys).
    ///
    /// # Safety
    ///
    /// * Must only be used on a struct returned from a call to `into_owned_property_sys`, without modification.
    /// * Must not be called more than once on a `sys::GDExtensionPropertyInfo` struct.
    pub(crate) unsafe fn free_owned_property_sys(info: sys::GDExtensionPropertyInfo) {
        // SAFETY: This function was called on a pointer returned from `into_owned_property_sys`, thus both `info.name` and
        // `info.hint_string` were created from calls to `into_owned_string_sys` on their respective types.
        // Additionally, this function isn't called more than once on a struct containing the same `name` or `hint_string` pointers.
        unsafe {
            let _name = StringName::from_owned_string_sys(info.name);
            let _hint_string = GString::from_owned_string_sys(info.hint_string);
        }
    }
}

// ----------------------------------------------------------------------------------------------------------------------------------------------

/// Info needed by Godot, for how to export a type to the editor.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct PropertyHintInfo {
    pub hint: PropertyHint,
    pub hint_string: GString,
}

impl PropertyHintInfo {
    /// Create a new `PropertyHintInfo` with a property hint of [`PROPERTY_HINT_NONE`](PropertyHint::NONE), and no hint string.
    pub fn none() -> Self {
        Self {
            hint: PropertyHint::NONE,
            hint_string: GString::new(),
        }
    }

    /// Use [`PROPERTY_HINT_NONE`](PropertyHint::NONE) with `T`'s Godot type name.
    ///
    /// Starting with Godot version 4.3, the hint string will always be the empty string. Before that, the hint string is set to
    /// be the Godot type name of `T`.
    pub fn type_name<T: GodotType>() -> Self {
        let type_name = T::godot_type_name();
        let hint_string = if sys::GdextBuild::since_api("4.3") {
            GString::new()
        } else {
            GString::from(type_name)
        };

        Self {
            hint: PropertyHint::NONE,
            hint_string,
        }
    }

    /// Use for `#[var]` properties -- [`PROPERTY_HINT_ARRAY_TYPE`](PropertyHint::ARRAY_TYPE) with the type name as hint string.
    pub fn var_array_element<T: ArrayElement>() -> Self {
        Self {
            hint: PropertyHint::ARRAY_TYPE,
            hint_string: GString::from(T::godot_type_name()),
        }
    }

    /// Use for `#[export]` properties -- [`PROPERTY_HINT_TYPE_STRING`](PropertyHint::TYPE_STRING) with the **element** type string as hint string.
    pub fn export_array_element<T: ArrayElement>() -> Self {
        Self {
            hint: PropertyHint::TYPE_STRING,
            hint_string: GString::from(T::element_type_string()),
        }
    }

    /// Use for `#[export]` properties -- [`PROPERTY_HINT_TYPE_STRING`](PropertyHint::TYPE_STRING) with the **element** type string as hint string.
    pub fn export_packed_array_element<T: PackedArrayElement>() -> Self {
        Self {
            hint: PropertyHint::TYPE_STRING,
            hint_string: GString::from(T::element_type_string()),
        }
    }
}
