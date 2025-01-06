use crate::resp::RESP;
use crate::value::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Get(String),
    Set(String, Value),
    Inc(String),
    Dec(String),
    Hello,
    Save,
    // User-side commands (the server should never see these)
    Help,
    Exit,
}

// TODO: Yes, this is super boilerplate-y. There's probably some way to generate all of these functions
// from the enum definition using a macro, but I haven't written it yet.

impl RESP for Command {
    fn encode_resp(self) -> String {
        match self {
            Self::Get(key) => Value::encode_resp(Value::Array(vec![
                Value::String("GET".to_string()),
                Value::String(key),
            ])),
            Self::Set(key, value) => Value::encode_resp(Value::Array(vec![
                Value::String("SET".to_string()),
                Value::String(key),
                value,
            ])),
            Self::Inc(key) => Value::encode_resp(Value::Array(vec![
                Value::String("INC".to_string()),
                Value::String(key),
            ])),
            Self::Dec(key) => Value::encode_resp(Value::Array(vec![
                Value::String("DEC".to_string()),
                Value::String(key),
            ])),
            Self::Hello => {
                Value::encode_resp(Value::Array(vec![Value::String("HELLO".to_string())]))
            }
            Self::Save => Value::encode_resp(Value::Array(vec![Value::String("SAVE".to_string())])),
            _ => unreachable!(),
        }
    }

    fn decode_resp(s: String) -> Result<Self, String> {
        let value = Value::decode_resp(s)?;
        let Value::Array(a) = value else {
            return Err("expected array of strings".to_string());
        };

        let Some(Value::String(cmd)) = a.first() else {
            return Err("expected array of strings".to_string());
        };

        match cmd.as_str() {
            "GET" => {
                let key = a.get(1).unwrap();
                Ok(Self::Get(key.to_string()))
            }
            "SET" => {
                let key = a.get(1).unwrap();
                let value = a.get(2).unwrap();
                Ok(Self::Set(key.to_string(), value.clone()))
            }
            "INC" => {
                let key = a.get(1).unwrap();
                Ok(Self::Inc(key.to_string()))
            }
            "DEC" => {
                let key = a.get(1).unwrap();
                Ok(Self::Dec(key.to_string()))
            }
            "HELLO" => Ok(Self::Hello),
            "SAVE" => Ok(Self::Save),
            _ => Err("unknown command".to_string()),
        }
    }
}
