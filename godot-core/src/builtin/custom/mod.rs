use std::fmt;
use std::fmt::Display;
use std::marker::PhantomData;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use serde::de::value::I64Deserializer;
use serde::de::{Error, Visitor};
use crate::builtin::{Dictionary, GString, Variant};
use crate::builtin::meta::{ConvertError, FromGodot, GodotConvert, ToGodot};
use crate::engine::{Area2D, IObject, Json, Object, StaticBody2D, Timer};
use crate::global::instance_from_id;
use crate::godot_print;
use crate::obj::{Gd, GodotClass, Inherits};

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

macro_rules! impl_serialize_deserialize {
    ($t: ident) => {

impl Serialize for Gd<$t>{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        serializer.serialize_str(&self.instance_id().to_string())
    }
}


impl<'de> Deserialize<'de> for Gd<$t> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        struct MyStructVisitor;

        impl<'de> Visitor<'de> for MyStructVisitor{
            type Value = Gd<$t>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("an i64")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> where E: Error {
                let value = v.parse::<i64>().unwrap();
                Ok(instance_from_id(value).unwrap().cast::<$t>())
            }
        }

        deserializer.deserialize_str(MyStructVisitor)
    }
}
    };
}


impl_serialize_deserialize!(Area2D);
impl_serialize_deserialize!(StaticBody2D);
impl_serialize_deserialize!(Timer);
