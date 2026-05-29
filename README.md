# xstream

[![crates.io](https://img.shields.io/crates/v/xstream-util)](https://crates.io/crates/xstream-util)
[![documentation](https://docs.rs/xstream-util/badge.svg)](https://docs.rs/xstream-util)
[![license](https://img.shields.io/github/license/erikbrinkman/xstream)](LICENSE)
[![tests](https://github.com/erikbrinkman/xstream/actions/workflows/rust.yml/badge.svg)](https://github.com/erikbrinkman/xstream/actions/workflows/rust.yml)

> **ℹ️ This project is deprecated and no longer maintained.**
>
> Its functionality is covered by [GNU `parallel`](https://www.gnu.org/software/parallel/),
> which is mature, widely packaged, and actively maintained. Prefer it for new work:
>
> | xstream | GNU `parallel` equivalent |
> |---------|---------------------------|
> | `xstream -- cmd` | `parallel --pipe -N1 cmd` |
> | `xstream -0 -- cmd` | `parallel --pipe --recend '\0' -N1 cmd` |
> | `xstream -p N -- cmd` | `parallel --pipe -N1 -j N cmd` |
> | `xstream --reuse -p N -- cmd` | `parallel --pipe --round-robin -j N cmd` |
>
> The repository remains available, read-only, for reference and existing users.

A command line tool to split a stream by a delimiter and pipe each section to a child process.

Each chunk can be piped to a new process, with limited parallelism, or for
embarassingly parallel processing, processes can be reused.

## Installation

```sh
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

## Other tools

- You may also be interested in [`xstdin`](https://github.com/patte/xstdin-rs), which is possibly more performant for the specific task of splitting a large input among several long running processes.
