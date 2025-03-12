use crate::DisplayableValue;
use cel_interpreter::objects::{Key, Map};
use cel_interpreter::Value;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct ExecutionContext {
    pub variables: PassableMap,
    pub expression: String,
    pub computed: Option<HashMap<String, Vec<PassableValue>>>,
    pub device: Option<HashMap<String, Vec<PassableValue>>>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct PassableMap {
    pub map: HashMap<String, PassableValue>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "value")]
pub enum PassableValue {
    #[serde(rename = "list")]
    List(Vec<PassableValue>),
    #[serde(rename = "map")]
    PMap(HashMap<String, PassableValue>),
    #[serde(rename = "function")]
    Function(String, Option<Box<PassableValue>>),
    #[serde(rename = "int")]
    Int(i64),
    #[serde(rename = "uint")]
    UInt(u64),
    #[serde(rename = "float")]
    Float(f64),
    #[serde(rename = "string")]
    String(String),
    #[serde(rename = "bytes")]
    Bytes(Vec<u8>),
    #[serde(rename = "bool")]
    Bool(bool),
    #[serde(rename = "timestamp")]
    Timestamp(i64),
    Null,
}

impl PartialEq for PassableValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (PassableValue::PMap(a), PassableValue::PMap(b)) => a == b,
            (PassableValue::List(a), PassableValue::List(b)) => a == b,
            (PassableValue::Function(a1, a2), PassableValue::Function(b1, b2)) => {
                a1 == b1 && a2 == b2
            }
            (PassableValue::Int(a), PassableValue::Int(b)) => a == b,
            (PassableValue::UInt(a), PassableValue::UInt(b)) => a == b,
            (PassableValue::Float(a), PassableValue::Float(b)) => a == b,
            (PassableValue::String(a), PassableValue::String(b)) => a == b,
            (PassableValue::Bytes(a), PassableValue::Bytes(b)) => a == b,
            (PassableValue::Bool(a), PassableValue::Bool(b)) => a == b,
            (PassableValue::Null, PassableValue::Null) => true,
            (PassableValue::Timestamp(a), PassableValue::Timestamp(b)) => a == b,
            // Allow different numeric types to be compared without explicit casting.
            (PassableValue::Int(a), PassableValue::UInt(b)) => a
                .to_owned()
                .try_into()
                .map(|a: u64| a == *b)
                .unwrap_or(false),
            (PassableValue::Int(a), PassableValue::Float(b)) => (*a as f64) == *b,
            (PassableValue::UInt(a), PassableValue::Int(b)) => a
                .to_owned()
                .try_into()
                .map(|a: i64| a == *b)
                .unwrap_or(false),
            (PassableValue::UInt(a), PassableValue::Float(b)) => (*a as f64) == *b,
            (PassableValue::Float(a), PassableValue::Int(b)) => *a == (*b as f64),
            (PassableValue::Float(a), PassableValue::UInt(b)) => *a == (*b as f64),
            (_, _) => false,
        }
    }
}

impl PassableValue {
    pub fn to_cel(&self) -> Value {
        match self {
            PassableValue::List(list) => {
                let mapped_list: Vec<Value> = list.iter().map(|item| item.to_cel()).collect();
                Value::List(Arc::new(mapped_list))
            }
            PassableValue::PMap(map) => {
                let mapped_map = map
                    .iter()
                    .map(|(k, v)| (Key::String(Arc::from(k.clone())), (*v).to_cel()))
                    .collect();
                Value::Map(Map {
                    map: Arc::new(mapped_map),
                })
            }
            PassableValue::Function(name, arg) => {
                let mapped_arg = arg.as_ref().map(|arg| arg.to_cel());
                Value::Function(Arc::from(name.clone()), mapped_arg.map(|v| Box::new(v)))
            }
            PassableValue::Int(i) => Value::Int(*i),
            PassableValue::UInt(u) => Value::UInt(*u),
            PassableValue::Float(f) => Value::Float(*f),
            PassableValue::String(s) => Value::String(Arc::from(s.clone())),
            PassableValue::Bytes(b) => Value::Bytes(Arc::from(b.clone())),
            PassableValue::Bool(b) => Value::Bool(*b),
            PassableValue::Timestamp(t) => Value::Int(*t),
            PassableValue::Null => Value::Null,
        }
    }
}

fn key_to_string(key: Key) -> String {
    match key {
        Key::String(s) => (*s).clone(),
        Key::Int(i) => i.to_string(),
        Key::Uint(u) => u.to_string(),
        Key::Bool(b) => b.to_string(),
    }
}
impl DisplayableValue {
    pub fn to_passable(&self) -> PassableValue {
        match &self.0 {
            Value::List(list) => {
                let mapped_list: Vec<PassableValue> = list
                    .iter()
                    .map(|item| DisplayableValue(item.clone()).to_passable())
                    .collect();
                PassableValue::List(mapped_list)
            }
            Value::Map(map) => {
                let mapped_map: HashMap<String, PassableValue> = map
                    .map
                    .iter()
                    .map(|(k, v)| {
                        (
                            key_to_string(k.clone()),
                            DisplayableValue(v.clone()).to_passable(),
                        )
                    })
                    .collect();
                PassableValue::PMap(mapped_map)
            }
            Value::Function(name, arg) => {
                let mapped_arg = arg.as_ref().map(|arg| {
                    let arg = *arg.clone();
                    let arg = DisplayableValue(arg).to_passable();
                    Box::new(arg)
                });
                PassableValue::Function((**name).clone(), mapped_arg)
            }
            Value::Int(i) => PassableValue::Int(*i),
            Value::UInt(u) => PassableValue::UInt(*u),
            Value::Float(f) => PassableValue::Float(*f),
            Value::String(s) => PassableValue::String((**s).clone()),
            Value::Bytes(b) => PassableValue::Bytes((**b).clone()),
            Value::Bool(b) => PassableValue::Bool(*b),
            Value::Duration(_) => PassableValue::Null,
            Value::Timestamp(t) => PassableValue::Timestamp(t.timestamp()),
            Value::Null => PassableValue::Null,
        }
    }
}
