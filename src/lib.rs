#![feature(plugin, custom_derive, custom_attribute, test, box_syntax)]
#![plugin(serde_macros)]

// Copyright 2016 drum Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

extern crate bincode;
extern crate serde;

use std::collections::BTreeMap;
use std::collections::btree_map::Keys;
use std::option::Option;
use std::io::*;
use std::marker::PhantomData;
use std::cell::RefCell;
use bincode::SizeLimit;
use serde::{Serialize, Deserialize};
use bincode::serde::{
  deserialize_from, serialize, serialize_into, serialized_size
};

pub trait KVStore {
  type Key;
  type Value;

  fn insert(&mut self, key: Self::Key, value: Self::Value) -> Result<()>;
  fn get(&self, key: &Self::Key) -> Result<Option<Self::Value>>;
  fn remove(&mut self, key: &Self::Key) -> Result<Option<Self::Value>>;
  fn keys<'a>(&'a self) -> Keys<'a, Self::Key, u64>;
}

#[derive(Serialize, Deserialize)]
struct Entry<K, V> {
  key: K,
  value: V
}

pub struct Store<K, V, F> {
  file: RefCell<F>,
  keys: BTreeMap<K, u64>,
  data: PhantomData<V>,
  offset: u64,
  entries: u64
}

impl<K, V, F> Store<K, V, F>
where K: Eq + Ord + Serialize + Deserialize,
      V: Serialize + Deserialize,
      F: Read + Write + Seek
{
  pub fn new(target: F) -> Store<K, V, F> {
    let header_size = serialized_size(&0u64);
    Store {
      file: RefCell::new(target),
      keys: BTreeMap::new(),
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
    let entries = deserialize_from::<F, u64>(&mut *file, SizeLimit::Infinite).unwrap_or(0);
    self.entries = entries;
    for _ in 0..entries {
      try!(file.seek(SeekFrom::Start(self.offset)));
      let entry = deserialize_from::<F, Entry<K, V>>(&mut *file, SizeLimit::Infinite).unwrap();
      let size = serialized_size(&entry) as i64;
      self.keys.insert(entry.key, self.offset);
      self.offset = self.offset + size as u64;
    }
    try!(file.seek(SeekFrom::End(0)));
    Ok(())
  }

  fn add_entry(&mut self) -> Result<()> {
    self.entries = self.entries + 1;
    let data = serialize(&self.entries, SizeLimit::Infinite);
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

  pub fn insert(&mut self, key: K, value: V) -> Result<()> {
    try!(self.add_entry());

    let entry = Entry {
      key: key,
      value: value
    };

    let mut file = self.file.borrow_mut();
    try!(file.seek(SeekFrom::End(0)));
    serialize_into(&mut *file, &entry, SizeLimit::Infinite).unwrap_or(());
    try!(file.flush());

    let start = self.offset;
    let size = serialized_size(&entry) as usize;
    self.offset = self.offset + size as u64;
    self.keys.insert(entry.key, start);
    Ok(())
  }

  pub fn get(&self, key: &K) -> Result<Option<V>> {
    let location = self.keys.get(key);
    match location {
      Some(loc) => {
        let mut file = self.file.borrow_mut();
        try!(file.seek(SeekFrom::Start(*loc)));
        let entry = deserialize_from::<F, Entry<K, V>>(&mut *file, SizeLimit::Infinite).unwrap();
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

impl<K, V, F> KVStore for Store<K, V, F>
where K: Eq + Ord + Serialize + Deserialize,
      V: Serialize + Deserialize,
      F: Read + Write + Seek
{
  type Key = K;
  type Value = V;

  fn insert(&mut self, key: Self::Key, value: Self::Value) -> Result<()> {
    self.insert(key, value)
  }

  fn get(&self, key: &Self::Key) -> Result<Option<Self::Value>> {
    self.get(key)
  }

  fn remove(&mut self, key: &Self::Key) -> Result<Option<Self::Value>> {
    self.remove(key)
  }

  fn keys<'a>(&'a self) -> Keys<'a, Self::Key, u64> {
    self.keys()
  }
}

#[cfg(test)]
mod tests {
  extern crate test;
  extern crate tempfile;

  use super::*;
  use std::io::*;
  use std::fs::File;
  use self::test::Bencher;
  use self::tempfile::tempfile;

  #[test]
  fn insert_get() {
    #[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
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

  #[bench]
  fn bench_insert(b: &mut Bencher) {
    let buffer: Cursor<Vec<u8>> = Cursor::new(Vec::new());
    let mut store: Store<String, u64, Cursor<Vec<u8>>> = Store::new(buffer);
    b.iter(|| {
      store.insert(String::from("foo"), 50).unwrap()
    });
  }

  #[bench]
  fn bench_get(b: &mut Bencher) {
    let buffer: Cursor<Vec<u8>> = Cursor::new(Vec::new());
    let mut store: Store<String, u64, Cursor<Vec<u8>>> = Store::new(buffer);
    store.insert(String::from("foo"), 50).unwrap();
    let key = String::from("foo");
    b.iter(|| {
      store.get(&key).unwrap().unwrap()
    });
  }

  #[bench]
  fn bench_file_insert(b: &mut Bencher) {
    let buffer = tempfile().unwrap();
    let mut store: Store<u64, u64, File> = Store::new(buffer);
    b.iter(|| {
      let _ = store.insert(10, 50);
    });
  }

  #[bench]
  fn bench_file_get(b: &mut Bencher) {
    let buffer = tempfile().unwrap();
    let mut store: Store<u64, u64, File> = Store::new(buffer);
    store.insert(10, 50).unwrap();
    b.iter(|| {
      let _ = store.get(&10);
    });
  }
}