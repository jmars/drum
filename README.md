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
- 810,000 reads/sec
- 480,000 writes/sec

## Memory
- 11,600,000 reads/sec
- 5,100,000 writes/sec

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
test tests::bench_file_get    ... bench:       1,236 ns/iter (+/- 157)
test tests::bench_file_insert ... bench:       2,092 ns/iter (+/- 304)
test tests::bench_get         ... bench:          86 ns/iter (+/- 6)
test tests::bench_insert      ... bench:         196 ns/iter (+/- 22)
```

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.