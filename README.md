# xstream

[![crates.io](https://img.shields.io/crates/v/xstream-util)](https://crates.io/crates/xstream-util)
[![documentation](https://docs.rs/xstream-util/badge.svg)](https://docs.rs/xstream-util)
[![license](https://img.shields.io/github/license/erikbrinkman/xstream)](LICENSE)
[![tests](https://github.com/erikbrinkman/xstream/actions/workflows/rust.yml/badge.svg)](https://github.com/erikbrinkman/xstream/actions/workflows/rust.yml)

A command line tool to split a stream by a delimiter and pipe each section to a child process.

Each chunk can be piped to a new process, with limited parallelism, or for
embarassingly parallel processing, processes can be reused.

## Installation

```
cargo install xstream-util
```

## Benchmarks

For a simple illustration of the speed up for reasonably sized streams, the following simple benchmark compares generating 1001 streams of integers and summing them with `bc`.

First, generate a null delimited set of streams with

```bash
time for I in {10000..11000}; do seq $I; echo -ne '0\0'; done
```

This stream is roughly 50M, making each stream roughly 50k.

I then piped this into `xstream` as
```bash
| time xstream -0 -w '' -- bash -c 'paste -sd+ | bc' > /dev/null
```
and `xargs` as
```bash
| time xargs -0I@ bash -c '<<< "@" head -n-1 | paste -sd+ | bc' > /dev/null
```

which on my system gives:

|  Program  |  User  | System | Elapsed |
|-----------|--------|--------|---------|
| `xstream` | 10.21s |  1.67s | 0:09.58 |
| `xargs`   | 15.72s |  2.85s | 0:14.52 |

This benchmark is a toy example, but `xstream` already provides a 30% speed up when each stream is only 50k.
