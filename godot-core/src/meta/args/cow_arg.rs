/*
 * Copyright (c) godot-rust; Bromeon and contributors.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use crate::builtin::Variant;
use crate::meta::error::ConvertError;
use crate::meta::{FromGodot, GodotConvert, GodotFfiVariant, RefArg, ToGodot};
use crate::sys;
use godot_ffi::{GodotFfi, GodotNullableFfi, PtrcallType};
use std::fmt;

/// Owned or borrowed value, used when passing arguments through `impl AsArg` to Godot APIs.
#[doc(hidden)]
#[derive(PartialEq)] // only for tests.
pub enum CowArg<'r, T> {
    Owned(T),
    Borrowed(&'r T),
}

impl<T> CowArg<'_, T> {
    pub fn cow_into_owned(self) -> T
    where
        T: Clone,
    {
        match self {
            CowArg::Owned(v) => v,
            CowArg::Borrowed(r) => r.clone(),
        }
    }

    pub fn cow_as_ref(&self) -> &T {
        match self {
            CowArg::Owned(v) => v,
            CowArg::Borrowed(r) => r,
        }
    }

    /// Returns the actual argument to be passed to function calls.
    ///
    /// [`CowArg`] does not implement [`AsArg<T>`] because a differently-named method is more explicit (fewer errors in codegen),
    /// and because [`AsArg::consume_arg()`] is not meaningful.
    pub fn cow_as_arg(&self) -> RefArg<'_, T> {
        RefArg::new(self.cow_as_ref())
    }
}

/// Exists for polymorphism in [`crate::meta::ParamType`].
///
/// Necessary for generics in e.g. `Array<T>`, when accepting `impl AsArg<T>` parameters.
///
/// `Borrow` may not be the most idiomatic trait for this, but it has the convenient feature that it's implemented for both `T` and `&T`.
/// Since this is a hidden API, it doesn't matter.
impl<T> std::borrow::Borrow<T> for CowArg<'_, T> {
    fn borrow(&self) -> &T {
        match self {
            CowArg::Owned(v) => v,
            CowArg::Borrowed(r) => r,
        }
    }
}

macro_rules! wrong_direction {
    ($fn:ident) => {
        unreachable!(concat!(
            stringify!($fn),
            ": CowArg should only be passed *to* Godot, not *from*."
        ))
    };
}

impl<T> GodotConvert for CowArg<'_, T>
where
    T: GodotConvert,
{
    type Via = T::Via;
}

impl<T> ToGodot for CowArg<'_, T>
where
    T: ToGodot,
{
    type ToVia<'v>
        = T::ToVia<'v>
    where
        Self: 'v;

    fn to_godot(&self) -> Self::ToVia<'_> {
        self.cow_as_ref().to_godot()
    }
}

// TODO refactor signature tuples into separate in+out traits, so FromGodot is no longer needed.
impl<T> FromGodot for CowArg<'_, T>
where
    T: FromGodot,
{
    fn try_from_godot(_via: Self::Via) -> Result<Self, ConvertError> {
        wrong_direction!(try_from_godot)
    }
}

impl<T> fmt::Debug for CowArg<'_, T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CowArg::Owned(v) => write!(f, "CowArg::Owned({v:?})"),
            CowArg::Borrowed(r) => write!(f, "CowArg::Borrowed({r:?})"),
        }
    }
}

// SAFETY: delegated to T.
unsafe impl<T> GodotFfi for CowArg<'_, T>
where
    T: GodotFfi,
{
    fn variant_type() -> sys::VariantType {
        T::variant_type()
    }

    unsafe fn new_from_sys(_ptr: sys::GDExtensionConstTypePtr) -> Self {
        wrong_direction!(new_from_sys)
    }

    unsafe fn new_with_uninit(_init_fn: impl FnOnce(sys::GDExtensionUninitializedTypePtr)) -> Self {
        wrong_direction!(new_with_uninit)
    }

    unsafe fn new_with_init(_init_fn: impl FnOnce(sys::GDExtensionTypePtr)) -> Self {
        wrong_direction!(new_with_init)
    }

    fn sys(&self) -> sys::GDExtensionConstTypePtr {
        self.cow_as_ref().sys()
    }

    fn sys_mut(&mut self) -> sys::GDExtensionTypePtr {
        unreachable!("CowArg::sys_mut() currently not used by FFI marshalling layer, but only by specific functions");
    }

    // This function must be overridden; the default delegating to sys() is wrong for e.g. RawGd<T>.
    // See also other manual overrides of as_arg_ptr().
    fn as_arg_ptr(&self) -> sys::GDExtensionConstTypePtr {
        self.cow_as_ref().as_arg_ptr()
    }

    unsafe fn from_arg_ptr(_ptr: sys::GDExtensionTypePtr, _call_type: PtrcallType) -> Self {
        wrong_direction!(from_arg_ptr)
    }

    unsafe fn move_return_ptr(self, _dst: sys::GDExtensionTypePtr, _call_type: PtrcallType) {
        // This one is implemented, because it's used for return types implementing ToGodot.
        unreachable!("Calling CowArg::move_return_ptr is a mistake, as CowArg is intended only for arguments. Use the underlying value type.");
    }
}

impl<T> GodotFfiVariant for CowArg<'_, T>
where
    T: GodotFfiVariant,
{
    fn ffi_to_variant(&self) -> Variant {
        self.cow_as_ref().ffi_to_variant()
    }

    fn ffi_from_variant(_variant: &Variant) -> Result<Self, ConvertError> {
        wrong_direction!(ffi_from_variant)
    }
}

impl<T> GodotNullableFfi for CowArg<'_, T>
where
    T: GodotNullableFfi,
{
    fn null() -> Self {
        CowArg::Owned(T::null())
    }

    fn is_null(&self) -> bool {
        self.cow_as_ref().is_null()
    }
}
