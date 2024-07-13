use std::fmt;
use std::fmt::Display;
use std::marker::PhantomData;
use anyhow::anyhow;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use serde::de::value::I64Deserializer;
use serde::de::{Error, Visitor};
use crate::builtin::{Array, Dictionary, GString, Variant};
use crate::builtin::meta::{ConvertError, FromGodot, GodotConvert, ToGodot};
use crate::classes::Texture2D;
use crate::engine::{Area2D, IObject, Json, Object, StaticBody2D, Timer};
use crate::global::{instance_from_id, is_instance_id_valid, PropertyHint};
use crate::godot_print;
use crate::obj::{Gd, GodotClass, Inherits};
use crate::registry::property::{Export, PropertyHintInfo, Var};

type Anyhow<T> = anyhow::Result<T>;

impl<T> GodotConvert for Anyhow<T> {
    type Via = Dictionary;
}



impl<T> ToGodot for  Anyhow<T>
where T: Serialize{
    fn to_godot(&self) -> Self::Via {
        let mut dict = Dictionary::new();
        dict.insert("Ok", "");
        dict.insert("Err", "");

        match self{
            Ok(s) => {
                match serde_json::to_string(s){
                    Ok(s) => {
                        let data = Json::parse_string(s.into());
                        dict.insert("Ok", data);
                        return dict
                    }
                    Err(e) => {
                        panic!("serde_json::to_string error: {}", e);
                    }
                }
            }
            Err(e) => {
                dict.insert("Err", e.to_string());
                return dict
            }
        }
    }
}

impl<T> FromGodot for Anyhow<T> {
    fn try_from_godot(via: Self::Via) -> Result<Self, ConvertError> {
        Err(ConvertError::new("not supported"))
    }
}


impl<T:GodotConvert<Via =GString> + ToGodot + FromGodot + Var> GodotConvert for Vec<T>
{
    type Via = Array<GString>;
}

impl<T:GodotConvert<Via =GString> + ToGodot + FromGodot + Var> ToGodot for Vec<T>{
    fn to_godot(&self) -> Self::Via {
        let mut array = Array::new();
        for x in self {
            array.push(x.to_godot());
        }
        array
    }
}

impl<T:GodotConvert<Via =GString> + ToGodot + FromGodot + Var> FromGodot for  Vec<T>{
    fn try_from_godot(via: Self::Via) -> Result<Self, ConvertError> {
        let mut ret = vec![];
        for x in via.iter_shared() {
            ret.push(T::from_godot(x));
        }
        Ok(ret)
    }
}

impl<T:GodotConvert<Via =GString> + ToGodot + FromGodot + Var> Var for  Vec<T>
{
    fn get_property(&self) -> Self::Via {
        ToGodot::to_godot(self)
    }

    fn set_property(&mut self, value: Self::Via) {
        *self = FromGodot::from_godot(value);
    }
    fn property_hint() -> PropertyHintInfo {

        PropertyHintInfo{
            hint: PropertyHint::ARRAY_TYPE,
            // "hint_string": str(TYPE_INT) + "/" + str(PROPERTY_HINT_ENUM) + ":" + ",".join(CustomEnum.keys())
            hint_string: format!("4/2:{}",  T::property_hint().hint_string.to_string()).into(),
        }
    }
}

impl<T:GodotConvert<Via =GString> + ToGodot + FromGodot + Var> Export for Vec<T>{
    fn default_export_info() -> PropertyHintInfo {
        Self::property_hint()
    }
}