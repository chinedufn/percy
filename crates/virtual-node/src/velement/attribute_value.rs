use std::fmt::{Display, Formatter};
use wasm_bindgen::JsValue;

/// The value associated with an element's attribute.
///
/// For <button disabled=true></button>, the element attribute value would be
/// `ElementAttributeValue::Bool(true)`
#[derive(Debug, PartialEq, Clone)]
pub enum AttributeValue {
    /// A string attribute such as value="My text input contents"
    String(String),
    /// A boolean attribute disabled=true
    Bool(bool),
}

impl AttributeValue {
    /// If the attribute is a string, return it. Otherwise return None.
    pub fn as_string(&self) -> Option<&String> {
        match self {
            AttributeValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// If the attribute is a bool, return it. Otherwise return None.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            AttributeValue::Bool(b) => Some(*b),
            _ => None,
        }
    }
}

// Implements
//   From<T> and From<&T> -> AttributeValue::String(T.to_string())
macro_rules! to_string_impls {
    ($ty:ty) => {
        impl From<$ty> for AttributeValue {
            fn from(val: $ty) -> Self {
                AttributeValue::String(val.to_string())
            }
        }

        impl From<&$ty> for AttributeValue {
            fn from(val: &$ty) -> Self {
                AttributeValue::String(val.to_string())
            }
        }
    };

    ($ty:ty, $($tys:ty),*) => {
        to_string_impls!( $ty );
        to_string_impls! ( $($tys),* );
    }
}
to_string_impls!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64);

impl Into<JsValue> for AttributeValue {
    fn into(self) -> JsValue {
        match self {
            AttributeValue::String(s) => s.into(),
            AttributeValue::Bool(b) => b.into(),
        }
    }
}

impl From<String> for AttributeValue {
    fn from(s: String) -> Self {
        AttributeValue::String(s)
    }
}

impl From<&String> for AttributeValue {
    fn from(s: &String) -> Self {
        AttributeValue::String(s.to_string())
    }
}

impl From<&str> for AttributeValue {
    fn from(s: &str) -> Self {
        AttributeValue::String(s.to_string())
    }
}

impl From<bool> for AttributeValue {
    fn from(b: bool) -> Self {
        AttributeValue::Bool(b)
    }
}

impl From<&bool> for AttributeValue {
    fn from(b: &bool) -> Self {
        AttributeValue::Bool(*b)
    }
}

impl Display for AttributeValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AttributeValue::String(s) => s.fmt(f),
            AttributeValue::Bool(b) => b.fmt(f),
        }
    }
}
