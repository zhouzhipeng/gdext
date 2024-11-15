use std::fmt::Display;
use crate::builtin::{Array, GString};
use crate::global::PropertyHint;
use crate::meta::{ArrayElement, AsArg, CowArg, FromGodot, GodotConvert, PropertyHintInfo, ToGodot};
use crate::meta::error::ConvertError;
use crate::registry::property::{Export, Var};


impl<T:GodotConvert<Via =GString> + ToGodot + FromGodot + Var> GodotConvert for Vec<T>
{
    type Via = Array<GString>;
}


impl<T:GodotConvert<Via =GString> + ToGodot + FromGodot + Var> ToGodot for Vec<T>{
    type ToVia<'v>= Self::Via where T: 'v;


    fn to_godot(&self) -> Self::Via {
        let mut array = Array::new();
        for x in self {
            array.push(&GString::from(x.to_variant().to_string()));

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
    fn var_hint() -> PropertyHintInfo {
        PropertyHintInfo{
            hint: PropertyHint::ARRAY_TYPE,
            // "hint_string": str(TYPE_INT) + "/" + str(PROPERTY_HINT_ENUM) + ":" + ",".join(CustomEnum.keys())
            hint_string: format!("4/2:{}",  T::var_hint().hint_string.to_string()).into(),
        }
    }
}

impl<T:GodotConvert<Via =GString> + ToGodot + FromGodot + Var> Export for Vec<T>{
    fn export_hint() -> PropertyHintInfo {

        Self::var_hint()
    }
}