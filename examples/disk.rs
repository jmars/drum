extern crate drum;
extern crate rustc_serialize;

use drum::*;
use std::io::*;
use std::collections::*;
use std::fs::{File, OpenOptions};

#[derive(PartialEq, Ord, Eq, PartialOrd, RustcEncodable, RustcDecodable)]
enum Value {
  Array(Vec<Value>),
  Object(BTreeMap<Value, Value>),
  String(String),
  Number(i64)
}

fn run() -> Result<()> {
  let file = try!(OpenOptions::new()
                  .read(true)
                  .write(true)
                  .create(true)
                  .append(true)
                  .open("test.db"));

  let mut store: Store<String, Value, File> = try!(Store::reopen(file));

  for key in store.keys() {
    println!("{}", key)
  }

  let previous = try!(store.get(&String::from("Hello WOrld")));

  try!(store.insert(
    String::from("Hello World"),
    Value::Array(vec![Value::Number(100)]))
  );

  match previous {
    Some(Value::Array(vec)) => {
      match vec[0] {
        Value::Number(num) => {
          println!("previous: {}", num);
        },

        _ => panic!()
      }
    },

    _ => ()
  }


  Ok(())
}

fn main() {
  run().unwrap();
  return;
}