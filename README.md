# Drum

[![Build Status](https://travis-ci.org/jmars/drum.svg?branch=master)](https://travis-ci.org/jmars/drum)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)
[![crates.io](http://meritbadge.herokuapp.com/drum)](https://crates.io/crates/drum)

> A BitCask inspired storage API for Rust.

Drum is 100% safe code:

```sh
$ ack unsafe src | wc
       0       0       0
```

```
running 9 tests
test tests::bench_insert ... ok
test tests::keys ... ok
test tests::insert_get ... ok
test tests::multiple_insert ... ok
test tests::bench_get ... ok
test tests::insert_remove ... ok
test tests::bench_file_get ... ok
test tests::reopen ... ok
test tests::bench_file_insert ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured
```

```
test tests::bench_file_get    ... bench:       2,430 ns/iter (+/- 413)
test tests::bench_file_insert ... bench:       6,160 ns/iter (+/- 1,424)
test tests::bench_get         ... bench:         113 ns/iter (+/- 32)
test tests::bench_insert      ... bench:         268 ns/iter (+/- 7)
```