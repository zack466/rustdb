use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    String(String),
    Int(i64),
    Array(Vec<Value>),
    Null,
    Ok,
} 

impl ToString for Value {
    fn to_string(&self) -> String {
        match self {
            Value::String(s) => s.clone(),
            Value::Ok => "OK".to_string(),
            _ => panic!("Value is not a string"),
        }
    }
}
