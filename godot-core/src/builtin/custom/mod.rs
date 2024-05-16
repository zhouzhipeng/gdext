use std::fmt::Display;
use serde::Serialize;
use crate::builtin::GString;
use crate::builtin::meta::{ConvertError, FromGodot, GodotConvert, ToGodot};

type Anyhow<T> = anyhow::Result<T>;

impl<T> GodotConvert for Anyhow<T> {
    type Via = GString;
}



impl<T> ToGodot for  Anyhow<T>
where T: Serialize{
    fn to_godot(&self) -> Self::Via {
        match self{
            Ok(s) => {
                serde_json::to_string(s).unwrap_or("ERR:string to json error!".to_string()).into()
            }
            Err(e) => {
                format!("ERR:{}", e.to_string()).into()
            }
        }
    }
}

impl<T> FromGodot for Anyhow<T> {
    fn try_from_godot(via: Self::Via) -> Result<Self, ConvertError> {
        Err(ConvertError::new("not supported"))
    }
}
