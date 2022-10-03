//! xstream library
//!
//! Provides a command to take a BufRead and split it as input among several processes. There's
//! current not any async support, and therefore no real way to interact with the processes
//! afterwards. Event collecting the output would require effort / synchronization, so currently
//! they're all just piped to the standard inhereted buffers.
#![warn(missing_docs)]

mod pool;
use pool::Pool;
use std::borrow::{Borrow, BorrowMut};
use std::io::{BufRead, Error, ErrorKind, Result, Write};
use std::process::{Command, Stdio};

/// stream one reader into several independent processes
///
/// ihandle will be delimited by delim, each section will be piped as stdin to a new command. Up to
/// max_parallel processes will exist at any one time, however if the commands are io intensive or
/// the buffer being piped to each command is long than there won't be much parallelization as this
/// must finish sending a section to a process before it will spin up another. This prevents excess
/// memory consumption, but will be slower than max parallelization.
///
/// Set max parallel to 0 to enable full parallelization.
pub fn xstream(
    mut command: impl BorrowMut<Command>,
    ihandle: &mut impl BufRead,
    delim: impl Borrow<[u8]>,
    max_parallel: usize,
) -> Result<()> {
    let mut pool = Pool::new(max_parallel);
    let command = command
        .borrow_mut()
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit());
    let delim = delim.borrow();

    while {
        let proc = pool.spawn(command)?;
        let ohandle = proc
            .stdin
            .as_mut()
            .ok_or_else(|| Error::new(ErrorKind::Other, "failed to capture child process stdin"))?;

        let mut new_process;
        while {
            let buf = ihandle.fill_buf()?;
            // TODO this takes worst case |buf| * |delim| when it only needs to take |buf|, but I
            // couln't find a builtin method to do it
            let (to_write, hit_delim) = match buf.windows(delim.len()).position(|w| w == delim) {
                // impossible to match delim
                _ if buf.len() < delim.len() => (buf.len(), false),
                // no match, write we can to guarantee we didn't write part of a match
                None => (buf.len() - delim.len() + 1, false),
                // matched write up to match, consume the match
                Some(pos) => (pos, true),
            };
            new_process = hit_delim;
            ohandle.write_all(&buf[..to_write])?;
            ihandle.consume(if hit_delim {
                to_write + delim.len()
            } else {
                to_write
            });
            !hit_delim && to_write > 0
        } {}

        new_process
    } {}

    pool.join()
}
