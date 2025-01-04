use crate::resp::RESP;
use crate::value::Value;
use crate::command::Command;

mod response;
mod resp;
mod value;
mod command;

fn main() {
    // let v = Value::Array(vec![
    //     Value::String("Asdf".to_string()),
    //     Value::Int(123),
    //     Value::Null,
    //     Value::Array(vec![
    //         Value::String("Hello".to_string()),
    //         Value::String("world".to_string()),
    //     ]),
    // ]);
    let v = Command::Set("Hello".to_string(), Value::String("world".to_string()));
    let encoded = v.clone().encode_resp();
    let decoded = Command::decode_resp(encoded).unwrap();
    println!("{:?}", decoded);
    assert!(decoded == v);
}

#[test]
fn test_table() {
    let mut table = table::Table::new();

    for i in 0..1000 {
        table.set(
            format!("string/{}", i),
            Value::String(format!("{}", i)),
        );
        table.set(format!("int/{}", i), Value::Int(i));
    }

    for i in 0..1000 {
        assert!(
            table.get(&format!("string/{}", i)) == Some(Value::String(format!("{}", i)))
        );
        assert!(table.get(&format!("int/{}", i)) == Some(Value::Int(i)));
    }

    let serialized = bincode::serialize(&table).unwrap();
    let deserialized: table::Table = bincode::deserialize(&serialized).unwrap();

    assert!(deserialized == table);
}
