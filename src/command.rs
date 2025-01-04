use crate::value::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Get(String),
    Set(String, Value),
    Inc(String),
    Dec(String),
    Hello,
}