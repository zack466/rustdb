use bincode::{serialize, deserialize};

mod table;

fn main() {
    let mut table = table::Table::new();
    for i in 0..10 {
        table.insert(format!("asdf/{}", i), table::Value::String(format!("asdf/{}", i)));
    }

    table.to_disk("test.bin").unwrap();

    let table2 = table::Table::from_disk("test.bin").unwrap();

    assert!(table2 == table);
}


#[test]
fn test_table() {
    let mut table = table::Table::new();

    for i in 0..1000 {
        table.insert(format!("string/{}", i), table::Value::String(format!("{}", i)));
        table.insert(format!("int/{}", i), table::Value::Int(i));
        table.insert(format!("float/{}", i), table::Value::Float(i as f64));
    }

    for i in 0..1000 {
        assert!(table.get(&format!("string/{}", i)) == Some(table::Value::String(format!("{}", i))));
        assert!(table.get(&format!("int/{}", i)) == Some(table::Value::Int(i)));
        assert!(table.get(&format!("float/{}", i)) == Some(table::Value::Float(i as f64)));
    }

    let serialized = bincode::serialize(&table).unwrap();
    let deserialized: table::Table = bincode::deserialize(&serialized).unwrap();

    assert!(deserialized == table);
}