use std::fmt::Display;
use serde::Serialize;
use crate::builtin::{Dictionary, GString, Variant};
use crate::builtin::meta::{ConvertError, FromGodot, GodotConvert, ToGodot};
use crate::engine::Json;
use crate::godot_print;

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
