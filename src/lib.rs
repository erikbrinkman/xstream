//! xstream library
//!
//! Provides a command to take a BufRead and split it as input among several processes. There's
//! current not any async support, and therefore no real way to interact with the processes
//! afterwards. Event collecting the output would require effort / synchronization, so currently
//! they're all just piped to the standard inhereted buffers.

mod pool;
use pool::Pool;
use std::borrow::BorrowMut;
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
    delim: u8,
    max_parallel: usize,
) -> Result<()> {
    let mut pool = Pool::new(max_parallel);
    let command = command
        .borrow_mut()
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit());

    while {
        let proc = pool.spawn(command)?;
        let ohandle = proc.stdin.as_mut().ok_or(Error::new(
            ErrorKind::Other,
            "failed to capture child process stdin",
        ))?;

        let mut hit_delim;
        while {
            let buf = ihandle.fill_buf()?;
            let mut itr = buf.splitn(2, |&c| c == delim);
            let dump = itr.next().unwrap(); // always get one
            hit_delim = itr.next().is_some();
            ohandle.write_all(dump)?;
            let size = dump.len();
            ihandle.consume(size + (hit_delim as usize));
            !hit_delim && size > 0
        } {}

        hit_delim
    } {}

    pool.join()
}
