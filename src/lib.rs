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
use bincode::rustc_serialize::*;

#[derive(RustcEncodable, RustcDecodable)]
struct Entry<K, V> {
  key: K,
  value: V
}

pub struct Store<K, V, F> {
  file: RefCell<F>,
  keys: HashMap<K, u64>,
  data: PhantomData<V>,
  offset: u64,
  entries: u64,
}

impl<K, V, F> Store<K, V, F>
where K: Eq + Hash + Encodable + Decodable,
      V: Encodable + Decodable,
      F: Read + Write + Seek
{
  pub fn new(target: F) -> Store<K, V, F> {
    let header_size = encoded_size(&0u64);
    Store {
      file: RefCell::new(target),
      keys: HashMap::new(),
      data: PhantomData,
      offset: header_size,
      entries: 0
    }
  } 

  pub fn reopen(target: F) -> Result<Store<K, V, F>> {
    let mut store: Store<K, V, F> = Store::new(target);
    try!(store.build_keys());
    Ok(store)
  }

  fn build_keys(&mut self) -> Result<()> {
    let mut file = self.file.borrow_mut();
    try!(file.seek(SeekFrom::Start(0)));
    let entries = decode_from::<F, u64>(&mut *file, SizeLimit::Infinite).unwrap_or(0);
    self.entries = entries;
    for _ in 0..entries {
      try!(file.seek(SeekFrom::Start(self.offset)));
      let entry = decode_from::<F, Entry<K, V>>(&mut *file, SizeLimit::Infinite).unwrap();
      let size = encoded_size(&entry) as i64;
      self.keys.insert(entry.key, self.offset);
      self.offset = self.offset + size as u64;
    }
    try!(file.seek(SeekFrom::End(0)));
    Ok(())
  }

  fn add_entry(&mut self) -> Result<()> {
    self.entries = self.entries + 1;
    let data = encode(&self.entries, SizeLimit::Infinite);
    match data {
      Ok(data) => {
        let mut file = self.file.borrow_mut();
        try!(file.seek(SeekFrom::Start(0)));
        try!(file.write_all(&data));
        try!(file.seek(SeekFrom::Start(self.offset)));
        Ok(())
      },
      Err(_) => Ok(())
    }
  }

  pub fn insert(&mut self, key: K, value: V) -> Result<Option<V>> {
    let previous = try!(self.get(&key));
    match previous {
      Some(_) => {},
      None => {
        let _ = self.add_entry();
      }
    }

    let entry = Entry {
      key: key,
      value: value
    };

    let mut file = self.file.borrow_mut();
    try!(file.seek(SeekFrom::End(0)));
    encode_into(&entry, &mut *file, SizeLimit::Infinite).unwrap_or(());

    let start = self.offset;
    let size = encoded_size(&entry) as usize;
    self.offset = self.offset + size as u64;
    self.keys.insert(entry.key, start);
    match previous {
      Some(prev) => Ok(Some(prev)),
      None => Ok(None)
    }
  }

  pub fn get(&self, key: &K) -> Result<Option<V>> {
    let location = self.keys.get(key);
    match location {
      Some(loc) => {
        let mut file = self.file.borrow_mut();
        try!(file.seek(SeekFrom::Start(*loc)));
        let entry = decode_from::<F, Entry<K, V>>(&mut *file, SizeLimit::Infinite).unwrap();
        try!(file.seek(SeekFrom::End(0)));
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

  pub fn keys<'a>(&'a self) -> Keys<'a, K, u64> {
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
  let mut store: Store<Vec<String>, Test, Cursor<Vec<u8>>> = Store::new(buffer);
  store.insert(vec![String::from("foo"), String::from("bar")], test.clone()).unwrap();
  let val = store.get(&vec![String::from("foo"), String::from("bar")]).unwrap().unwrap();
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
fn multiple_insert() {
  let buffer: Cursor<Vec<u8>> = Cursor::new(Vec::new());
  let mut store: Store<String, u64, Cursor<Vec<u8>>> = Store::new(buffer);
  store.insert(String::from("foo"), 100).unwrap();
  store.insert(String::from("bar"), 200).unwrap();
  assert_eq!(store.get(&String::from("foo")).unwrap().unwrap(), 100);
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

#[test]
fn reopen() {
  let mut buffer: Cursor<Vec<u8>> = Cursor::new(Vec::new());
  {
    let mut store: Store<String, u64, Cursor<Vec<u8>>> = Store::new(buffer);
    store.insert(String::from("foo"), 50).unwrap();
    buffer = store.file.into_inner();
  }

  let store: Store<String, u64, Cursor<Vec<u8>>> = Store::reopen(buffer).unwrap();

  for key in store.keys() {
    assert_eq!(*key, String::from("foo"));
  };

  assert_eq!(store.get(&String::from("foo")).unwrap().unwrap(), 50);
}