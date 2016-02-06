#![feature(plugin, custom_derive, custom_attribute)]
#![plugin(serde_macros)]
extern crate drum;
extern crate serde;
extern crate bufstream;

use drum::*;
use std::io::*;
use std::collections::*;
use std::fs::{OpenOptions};
use bufstream::BufStream;

#[derive(PartialEq, Ord, Eq, PartialOrd, Serialize, Deserialize)]
enum Value {
  Array(Vec<Value>),
  Object(BTreeMap<Value, Value>),
  String(String),
  Number(i64)
}

fn run() -> Result<()> {
  let msg = "Hello World";
  let file =
    BufStream::new(
      try!(OpenOptions::new()
      .read(true)
      .write(true)
      .create(true)
      .append(true)
      .open("test.db")));

  let mut store = try!(Store::reopen(file));

  for key in store.keys() {
    println!("{}", key)
  }

  let previous = try!(store.get(&String::from(msg)));

  try!(store.insert(
    String::from(msg),
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