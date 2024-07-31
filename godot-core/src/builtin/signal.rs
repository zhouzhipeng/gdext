/*
 * Copyright (c) godot-rust; Bromeon and contributors.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::fmt;
use std::ptr;

use godot_ffi as sys;

use crate::builtin::{inner, Array, Callable, Dictionary, StringName, Variant};
use crate::classes::Object;
use crate::global::Error;
use crate::meta::{FromGodot, GodotType, ToGodot};
use crate::obj::bounds::DynMemory;
use crate::obj::{Bounds, Gd, GodotClass, InstanceId};
use sys::{ffi_methods, GodotFfi};

/// A `Signal` represents a signal of an Object instance in Godot.
///
/// Signals are composed of a reference to an `Object` and the name of the signal on this object.
///
/// # Godot docs
///
/// [`Signal` (stable)](https://docs.godotengine.org/en/stable/classes/class_signal.html)
pub struct Signal {
    opaque: sys::types::OpaqueSignal,
}

impl Signal {
    fn from_opaque(opaque: sys::types::OpaqueSignal) -> Self {
        Self { opaque }
    }

    /// Create a signal for the signal `object::signal_name`.
    ///
    /// _Godot equivalent: `Signal(Object object, StringName signal)`_
    pub fn from_object_signal<T, S>(object: &Gd<T>, signal_name: S) -> Self
    where
        T: GodotClass,
        S: Into<StringName>,
    {
        let signal_name = signal_name.into();
        unsafe {
            Self::new_with_uninit(|self_ptr| {
                let ctor = sys::builtin_fn!(signal_from_object_signal);
                let raw = object.to_ffi();
                let args = [raw.as_arg_ptr(), signal_name.sys()];
                ctor(self_ptr, args.as_ptr());
            })
        }
    }

    /// Creates an invalid/empty signal that cannot be called.
    ///
    /// _Godot equivalent: `Signal()`_
    pub fn invalid() -> Self {
        unsafe {
            Self::new_with_uninit(|self_ptr| {
                let ctor = sys::builtin_fn!(signal_construct_default);
                ctor(self_ptr, ptr::null_mut())
            })
        }
    }

    /// Connects this signal to the specified callable.
    ///
    /// Optional flags can be also added to configure the connection's behavior (see [`ConnectFlags`](crate::classes::object::ConnectFlags) constants).
    /// You can provide additional arguments to the connected callable by using `Callable::bind`.
    ///
    /// A signal can only be connected once to the same [`Callable`]. If the signal is already connected,
    /// returns [`Error::ERR_INVALID_PARAMETER`] and
    /// pushes an error message, unless the signal is connected with [`ConnectFlags::REFERENCE_COUNTED`](crate::classes::object::ConnectFlags::REFERENCE_COUNTED).
    /// To prevent this, use [`Self::is_connected`] first to check for existing connections.
    pub fn connect(&self, callable: Callable, flags: i64) -> Error {
        let error = self.as_inner().connect(callable, flags);

        Error::from_godot(error as i32)
    }

    /// Disconnects this signal from the specified [`Callable`].
    ///
    /// If the connection does not exist, generates an error. Use [`Self::is_connected`] to make sure that the connection exists.
    pub fn disconnect(&self, callable: Callable) {
        self.as_inner().disconnect(callable);
    }

    /// Emits this signal.
    ///
    /// All Callables connected to this signal will be triggered.
    pub fn emit(&self, varargs: &[Variant]) {
        let Some(mut object) = self.object() else {
            return;
        };

        object.emit_signal(self.name(), varargs);
    }

    /// Returns an [`Array`] of connections for this signal.
    ///
    /// Each connection is represented as a Dictionary that contains three entries:
    ///  - `signal` is a reference to this [`Signal`];
    ///  - `callable` is a reference to the connected [`Callable`];
    ///  - `flags` is a combination of [`ConnectFlags`](crate::classes::object::ConnectFlags).
    ///
    /// _Godot equivalent: `get_connections`_
    pub fn connections(&self) -> Array<Dictionary> {
        self.as_inner()
            .get_connections()
            .iter_shared()
            .map(|variant| variant.to())
            .collect()
    }

    /// Returns the name of the signal.
    pub fn name(&self) -> StringName {
        self.as_inner().get_name()
    }

    /// Returns the object to which this signal belongs.
    ///
    /// Returns [`None`] when this signal doesn't have any object.
    ///
    /// _Godot equivalent: `get_object`_
    pub fn object(&self) -> Option<Gd<Object>> {
        self.as_inner().get_object().map(|mut object| {
            <Object as Bounds>::DynMemory::maybe_inc_ref(&mut object.raw);
            object
        })
    }

    /// Returns the ID of this signal's object, see also [`Gd::instance_id`].
    ///
    /// Returns [`None`] when this signal doesn't have any object.
    ///
    /// _Godot equivalent: `get_object_id`_
    pub fn object_id(&self) -> Option<InstanceId> {
        let id = self.as_inner().get_object_id();
        InstanceId::try_from_i64(id)
    }

    /// Returns `true` if the specified [`Callable`] is connected to this signal.
    pub fn is_connected(&self, callable: Callable) -> bool {
        self.as_inner().is_connected(callable)
    }

    /// Returns `true` if the signal's name does not exist in its object, or the object is not valid.
    pub fn is_null(&self) -> bool {
        self.as_inner().is_null()
    }

    #[doc(hidden)]
    pub fn as_inner(&self) -> inner::InnerSignal {
        inner::InnerSignal::from_outer(self)
    }
}

// SAFETY:
// The `opaque` in `Signal` is just a pair of pointers, and requires no special initialization or cleanup
// beyond what is done in `from_opaque` and `drop`. So using `*mut Opaque` is safe.
unsafe impl GodotFfi for Signal {
    fn variant_type() -> sys::VariantType {
        sys::VariantType::SIGNAL
    }

    ffi_methods! { type sys::GDExtensionTypePtr = *mut Opaque;
        fn new_from_sys;
        fn new_with_uninit;
        fn from_arg_ptr;
        fn sys;
        fn sys_mut;
        fn move_return_ptr;
    }

    unsafe fn new_with_init(init_fn: impl FnOnce(sys::GDExtensionTypePtr)) -> Self {
        let mut result = Self::invalid();
        init_fn(result.sys_mut());
        result
    }
}

impl_builtin_traits! {
    for Signal {
        Clone => signal_construct_copy;
        Drop => signal_destroy;
        PartialEq => signal_operator_equal;
    }
}

crate::meta::impl_godot_as_self!(Signal);

impl fmt::Debug for Signal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = self.name();
        let object = self.object();

        f.debug_struct("signal")
            .field("name", &name)
            .field("object", &object)
            .finish()
    }
}

impl fmt::Display for Signal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_variant())
    }
}
