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

# Overview

Drum is an attempt to write a bitcask style key-value store entirely in rust.

# Performance

See the bottom of the readme for benchmarks, so far the numbers look like:

## Disk
- 740,000 reads/sec
- 338,000 writes/sec

## Memory
- 8,700,000 reads/sec
- 3,800,000 writes/sec

### Why are writes so much slower?

The insert method returns the previously stored value, so we have to do a read
and then the write. No way to avoid this without changing the API.

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
test tests::bench_file_get    ... bench:       1,350 ns/iter (+/- 323)
test tests::bench_file_insert ... bench:       2,958 ns/iter (+/- 782)
test tests::bench_get         ... bench:         114 ns/iter (+/- 2)
test tests::bench_insert      ... bench:         260 ns/iter (+/- 113)
```