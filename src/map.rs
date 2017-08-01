use std::fmt;
use std::ops::{Deref, DerefMut, Index};
use std::iter::FromIterator;
use std::hash::Hash;
use std::borrow::Borrow;
use serde;
use serde::de::Error;
use serde_json::{self, Value};
use unreachable::unreachable;

#[derive(Default, Clone, PartialEq)]
pub struct Map(Value);

impl Map {
    pub fn into_inner(self) -> serde_json::Map<String, Value> {
        match self.0 {
            Value::Object(map) => map,
            _ => unsafe { unreachable() },
        }
    }

    pub fn into_value(self) -> Value {
        self.0
    }

    pub fn value_ref(&self) -> &Value {
        &self.0
    }

    pub fn value_mut(&mut self) -> &mut Value {
        &mut self.0
    }

    pub fn deserialize_into<'de, T: serde::Deserialize<'de>>(&'de self) -> Result<T, serde_json::Error> {
        T::deserialize(&self.0)
    }

    pub fn serialize_from<T: serde::Serialize>(t: T) -> Result<Self, serde_json::Error> {
        serde_json::value::to_value(t).and_then(Self::from_value)
    }

    fn from_value<E: Error>(v: Value) -> Result<Self, E> {
        let expected = &"object map";
        match v {
            map @ Value::Object(..) => Ok(Map(map)),
            Value::Null => Err(E::invalid_type(serde::de::Unexpected::Unit, expected)),
            Value::Bool(v) => Err(E::invalid_type(serde::de::Unexpected::Bool(v), expected)),
            Value::Number(v) => Err(if let Some(v) = v.as_u64() {
                E::invalid_type(serde::de::Unexpected::Unsigned(v), expected)
            } else if let Some(v) = v.as_i64() {
                E::invalid_type(serde::de::Unexpected::Signed(v), expected)
            } else if let Some(v) = v.as_f64() {
                E::invalid_type(serde::de::Unexpected::Float(v), expected)
            } else {
                E::invalid_type(serde::de::Unexpected::Other("number"), expected)
            }),
            Value::String(v) => Err(E::invalid_type(serde::de::Unexpected::Str(&v), expected)),
            Value::Array(..) => Err(E::invalid_type(serde::de::Unexpected::Seq, expected)),
        }
    }
}

impl Deref for Map {
    type Target = serde_json::Map<String, Value>;

    fn deref(&self) -> &Self::Target {
        match &self.0 {
            &Value::Object(ref map) => map,
            _ => unsafe { unreachable() },
        }
    }
}

impl DerefMut for Map {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match &mut self.0 {
            &mut Value::Object(ref mut map) => map,
            _ => unsafe { unreachable() },
        }
    }
}

impl From<serde_json::Map<String, Value>> for Map {
    fn from(m: serde_json::Map<String, Value>) -> Self {
        Map(Value::Object(m))
    }
}

impl From<Map> for Value {
    fn from(m: Map) -> Self {
        m.into_value()
    }
}

impl From<Map> for serde_json::Map<String, Value> {
    fn from(m: Map) -> Self {
        m.into_inner()
    }
}

impl fmt::Debug for Map {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.0, fmt)
    }
}

impl serde::Serialize for Map {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        serde::Serialize::serialize(&self.0, s)
    }
}

impl<'de> serde::Deserialize<'de> for Map {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        match serde::Deserialize::deserialize(d) {
            Ok(value) => Self::from_value(value),
            Err(err) => Err(err),
        }
    }
}

impl FromIterator<(String, Value)> for Map {
    fn from_iter<T: IntoIterator<Item=(String, Value)>>(iter: T) -> Self {
        serde_json::Map::from_iter(iter).into()
    }
}

impl Extend<(String, Value)> for Map {
    fn extend<T: IntoIterator<Item=(String, Value)>>(&mut self, iter: T) {
        self.deref_mut().extend(iter)
    }
}

impl IntoIterator for Map {
    type Item = (String, Value);
    type IntoIter = serde_json::map::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.into_inner().into_iter()
    }
}

impl<'a> IntoIterator for &'a Map {
    type Item = (&'a String, &'a Value);
    type IntoIter = serde_json::map::Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.deref().into_iter()
    }
}

impl<'a> IntoIterator for &'a mut Map {
    type Item = (&'a String, &'a mut Value);
    type IntoIter = serde_json::map::IterMut<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.deref_mut().into_iter()
    }
}

impl<'a, Q: Ord + Eq + Hash + ?Sized> Index<&'a Q> for Map where String: Borrow<Q> {
    type Output = Value;

    fn index(&self, index: &Q) -> &Self::Output {
        self.deref().index(index)
    }
}
// TODO: Index
