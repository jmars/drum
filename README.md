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

# Performance

See the bottom of the readme for benchmarks, so far the numbers look like:

## Disk
- 700,000 reads/sec
- 500,000 writes/sec

## Memory
- 8,900,000 reads/sec
- 5,500,000 writes/sec

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
test tests::bench_file_get    ... bench:       1,409 ns/iter (+/- 309)
test tests::bench_file_insert ... bench:       2,001 ns/iter (+/- 408)
test tests::bench_get         ... bench:         112 ns/iter (+/- 1)
test tests::bench_insert      ... bench:         182 ns/iter (+/- 23)
```