use crate::builtin::GString;
use crate::builtin::meta::{ConvertError, FromGodot, GodotConvert, ToGodot};

type Anyhow<T> = anyhow::Result<T>;

impl<T> GodotConvert for Anyhow<T> {
    type Via = GString;
}



impl<T> ToGodot for  Anyhow<T>{
    fn to_godot(&self) -> Self::Via {
        "anyhowtest".into()
    }
}

impl<T> FromGodot for Anyhow<T> {
    fn try_from_godot(via: Self::Via) -> Result<Self, ConvertError> {
        Err(ConvertError::new("not supported"))
    }
}
