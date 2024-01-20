# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.0.0] - 2024-01-20

### Added

- a rotationg process queue, instead of spawning a new process for every run,
  it will round-robin inputs between processes.
- the ability to change the write delimiter, e.g. the input can be null
  terminated, but still passed into the processes with a new line.
- exposure of the underlying `Pool` trait and the two implementations now used.
- a changelog

### Changed

- switched library to use its own error type instead of `std::io::Error`.
- delimiter behavior, before the delimiter wasn't written to child processes,
  now it is by default, the old behavior can be achieved by setting
  `write_delimiter` to `Some(b"")`.
- a trailing delimiter will now not cause a final process to run.
- changed delimiter trait from `Borrow<[u8]>` to `AsRef<[u8]>` permitting more
  types.
