extern crate bincode;
extern crate rustc_serialize;

use std::collections::HashMap;
use std::collections::hash_map::Keys;
use std::option::Option;
use std::io::*;
use std::marker::PhantomData;
use std::hash::Hash;
use std::cell::RefCell;
use rustc_serialize::{Encodable, Decodable};
use bincode::SizeLimit;
use bincode::rustc_serialize::{encode_into, encoded_size, decode};

#[derive(RustcEncodable, RustcDecodable)]
struct Entry<K, V> {
  key: K,
  value: V
}

pub struct Location {
  start: u64,
  size: usize
}

pub struct Store<K, V, F> {
  file: RefCell<F>,
  keys: HashMap<K, Location>,
  data: PhantomData<V>,
  buffer: RefCell<Option<Vec<u8>>>,
  offset: u64
}

impl<K, V, F> Store<K, V, F>
where K: Eq + Hash + Encodable + Decodable,
      V: Encodable + Decodable,
      F: Read + Write + Seek
{
  pub fn new(target: F) -> Store<K, V, F> {
    Store {
      file: RefCell::new(target),
      keys: HashMap::new(),
      data: PhantomData,
      buffer: RefCell::new(Some(Vec::new())),
      offset: 0
    }
  } 

  pub fn insert(&mut self, key: K, value: V) -> Result<Option<V>> {
    let previous = try!(self.get(&key));
    let entry = Entry {
      key: key,
      value: value
    };
    let mut file = self.file.borrow_mut();
    let buf = self.buffer.borrow_mut().take().unwrap();
    let mut encoded = Cursor::new(buf);
    encode_into(&entry, &mut encoded, SizeLimit::Infinite).unwrap();
    let buf = encoded.into_inner();
    let start = self.offset;
    let size = encoded_size(&entry) as usize;
    self.offset = self.offset + size as u64;
    // TODO put buffer back on fail
    try!(file.write_all(&buf[..size]));
    self.keys.insert(entry.key, Location { start: start, size: size });
    *self.buffer.borrow_mut() = Some(buf);
    match previous {
      Some(prev) => Ok(Some(prev)),
      None => Ok(None)
    }
  }

  pub fn get(&self, key: &K) -> Result<Option<V>> {
    let location = self.keys.get(key);
    match location {
      Some(loc) => {
        let mut buffer = self.buffer.borrow_mut();
        let buf = &mut buffer.as_mut().unwrap()[..loc.size];
        let mut file = self.file.borrow_mut();
        try!(file.seek(SeekFrom::Start(loc.start)));
        try!(file.read(buf));
        try!(file.seek(SeekFrom::End(0)));
        let entry: Entry<K, V> = decode(buf).unwrap();
        Ok(Some(entry.value))
      }
      None => Ok(None)
    }
  }

  pub fn remove(&mut self, key: &K) -> Result<Option<V>> {
    let previous = try!(self.get(key));
    match previous {
      Some(prev) => {
        self.keys.remove(key);
        Ok(Some(prev))
      }
      None => Ok(None)
    }
  }

  pub fn keys<'a>(&'a self) -> Keys<'a, K, Location> {
    self.keys.keys()
  }
}

#[test]
fn insert_get() {
  #[derive(RustcEncodable, RustcDecodable, PartialEq, Debug, Clone)]
  struct Test {
    num: u64,
    string: String
  }
  let test = Test {
    num: 42,
    string: String::from("testing")
  };
  let buffer: Cursor<Vec<u8>> = Cursor::new(Vec::new());
  let mut store: Store<String, Test, Cursor<Vec<u8>>> = Store::new(buffer);
  store.insert(String::from("foo"), test.clone()).unwrap();
  let val = store.get(&String::from("foo")).unwrap().unwrap();
  assert_eq!(test, val);
}

#[test]
fn insert_remove() {
  let buffer: Cursor<Vec<u8>> = Cursor::new(Vec::new());
  let mut store: Store<String, u64, Cursor<Vec<u8>>> = Store::new(buffer);
  store.insert(String::from("foo"), 100).unwrap();
  assert_eq!(store.get(&String::from("foo")).unwrap().unwrap(), 100);
  store.remove(&String::from("foo")).unwrap();
  match store.get(&String::from("foo")).unwrap() {
    None => assert!(true),
    Some(_) => assert!(false)
  }
}

#[test]
fn keys() {
  let buffer: Cursor<Vec<u8>> = Cursor::new(Vec::new());
  let mut store: Store<String, u64, Cursor<Vec<u8>>> = Store::new(buffer);
  store.insert(String::from("foo"), 50).unwrap();
  for key in store.keys() {
    assert_eq!(*key, String::from("foo"));
  }
}