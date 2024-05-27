/*
 * Copyright (c) godot-rust; Bromeon and contributors.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub use super::register::property::{Export, TypeStringHint, Var};

// Re-export macros.
pub use super::register::{godot_api, Export, GodotClass, GodotConvert, Var};

pub use super::builtin::__prelude_reexport::*;
pub use super::builtin::math::FloatExt as _;
pub use super::builtin::meta::{FromGodot, ToGodot, GodotConvert as GodotConvertTrait };

pub use super::engine::{
    AudioStreamPlayer, Camera2D, Camera3D, IAudioStreamPlayer, ICamera2D, ICamera3D, INode,
    INode2D, INode3D, IObject, IPackedScene, IRefCounted, IResource, ISceneTree, Input, Node,
    Node2D, Node3D, Object, PackedScene, RefCounted, Resource, SceneTree,
};
pub use super::extras::{GFile, IoError};
pub use super::global::{
    godot_error, godot_print, godot_print_rich, godot_script_error, godot_warn, load, save,
    try_load, try_save,
};

pub use super::init::{gdextension, ExtensionLibrary, InitLevel};
pub use super::obj::{Base, Gd, GdMut, GdRef, GodotClass, Inherits, InstanceId, OnReady};

// Make trait methods available.
pub use super::obj::EngineBitfield as _;
pub use super::obj::EngineEnum as _;
pub use super::obj::NewAlloc as _;
pub use super::obj::NewGd as _;
pub use super::obj::WithBaseField as _; // base(), base_mut(), to_gd()
