use crate::value::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum Response {
    Success(Value),
    Error(String),
}