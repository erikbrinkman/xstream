# xstream

A command line tool to split a stream by a delimiter and pipe each section to a child process.

## Installation

```
cargo install xstream-util
```

## Benchmarks

For a simple illustration of the speed up for reasonably sized streams, the following simple benchmark compares generating 1001 streams of integers and summing them with `bc`.

First, generate a null delimited set of streams with

```bash
time for I in {10000..11000}; do seq $I; echo -ne '\0'; done
```

This stream is roughly 50M, making each stream roughly 50k.

I then piped this into `xstream` as
```bash
| time xstream -0 -- bash -c 'paste -sd+ | bc' > /dev/null
```
and `xargs` as
```bash
| time xargs -0I@ bash -c '<<< "@" head -n-1 | paste -sd+ | bc' > /dev/null
```

which on my system gives:

|  Program  | User  | System | Elapsed |
|-----------|-------|--------|---------|
| `xstream` |  6.55 |   1.53 | 0:06.91 |
| `xargs`   | 17.26 |   3.98 | 0:18.01 |

This benchmark is a toy example, but `xstream` already provides a 60% speed up when each stream is only 50k.
