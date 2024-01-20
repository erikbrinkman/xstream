//! xstream library
//!
//! Provides [[xstream]] to take a `BufRead` and splits it as input among several processes.
//! There's current not any async support, and therefore no real way to interact with the processes
//! afterwards. Even collecting the output would require effort / synchronization, so currently
//! they're all just piped to the standard inhereted buffers.
//!
//! # Usage
//!
//! ```
//! use std::process::Command;
//! use xstream_util::Limiting;
//! # use std::io::BufReader;
//!
//! let mut input = // ...
//! # BufReader::new(&[0_u8; 0][..]);
//! // Spawn up to two `cat` processes, could also use `Rotating`
//! let mut pool = Limiting::new(Command::new("cat"), 2);
//! xstream_util::xstream(&mut pool, &mut input, &b"\n", &None::<&[u8]>).unwrap();
//! ```
#![warn(missing_docs)]
#![warn(clippy::pedantic)]

mod limit;
mod pool;
mod rot;

pub use limit::Limiting;
pub use pool::{Error, Pool};
pub use rot::Rotating;
use std::io::{BufRead, Write};

/// Stream one reader into several independent processes
///
/// `in_handle` will be delimited by `delim`, each section will be piped as stdin to a command spawned from `pool`.
///
/// # Notes
///
/// If the commands are io intensive or the buffer being piped to each command is long than there
/// won't be much parallelization as this must finish sending a section to a process before it will
/// spin up another.
///
/// # Errors
///
/// If there are problems spawning processes, the processes themselves fail, or there are problems
/// reading or writing to the available readers / writers.
pub fn xstream(
    pool: &mut impl Pool,
    in_handle: &mut impl BufRead,
    delim: impl AsRef<[u8]>,
    write_delim: &Option<impl AsRef<[u8]>>,
) -> Result<(), Error> {
    let delim = delim.as_ref();

    while !in_handle.fill_buf().map_err(Error::Input)?.is_empty() {
        let proc = pool.get()?;
        let out_handle = proc.stdin.as_mut().ok_or(Error::StdinNotPiped)?;

        while {
            let buf = in_handle.fill_buf().map_err(Error::Input)?;
            let (consume, hit_delim) = match buf.windows(delim.len()).position(|w| w == delim) {
                // no match
                None => (
                    if buf.len() < delim.len() {
                        // buffer can never contain the match, so dump the rest
                        buf.len()
                    } else {
                        // write we can to guarantee we didn't write part of a match
                        buf.len() - delim.len() + 1
                    },
                    false,
                ),
                // matched write up to match, consume the match
                Some(pos) => (pos + delim.len(), true),
            };
            if let (Some(wdel), true) = (write_delim, hit_delim) {
                out_handle
                    .write_all(&buf[..consume - delim.len()])
                    .map_err(Error::Output)?;
                out_handle.write_all(wdel.as_ref()).map_err(Error::Output)?;
            } else {
                out_handle
                    .write_all(&buf[..consume])
                    .map_err(Error::Output)?;
            }
            in_handle.consume(consume);
            !hit_delim && consume > 0
        } {}
    }

    pool.join()
}
