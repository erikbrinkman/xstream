//! A trait for a generic process pool used by xstream
use std::borrow::BorrowMut;
use std::error;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::io;
use std::process::Child;

/// Internal function to wait for a process
///
/// This will error in the event that it doesn't complete successfully (non-zero error code or
/// otherwise)
pub fn wait_proc(mut proc: impl BorrowMut<Child>) -> Result<(), Error> {
    let status = proc.borrow_mut().wait().map_err(Error::Wait)?;
    match status.code() {
        Some(0) => Ok(()),
        Some(code) => Err(Error::NonZeroExitCode(code)),
        None => Err(Error::KilledBySignal),
    }
}

/// An error raised by `xstream`
#[non_exhaustive]
#[derive(Debug)]
pub enum Error {
    /// The stdin to a child process wasn't piped
    StdinNotPiped,
    /// One of the spawned processes was killed by a signal
    KilledBySignal,
    /// One of the spawned processes returned a non-zero exit code
    NonZeroExitCode(i32),
    /// An error occured while trying to read from the input
    Input(io::Error),
    /// An error occured while trying to write to a child process
    Output(io::Error),
    /// An error occured while trying to spawn a child process
    Spawn(io::Error),
    /// An error occured while trying to wait for a child process
    Wait(io::Error),
}

impl Display for Error {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl error::Error for Error {}

/// A type that can `get` child processes on demand
pub trait Pool {
    /// Fetch a process from the pool
    ///
    /// Depending on the type of `Pool`, this may spawn a new process or just return one that is
    /// already running.
    ///
    /// # Errors
    ///
    /// When anything goes wrong when trying to create a new process.
    fn get(&mut self) -> Result<&mut Child, Error>;

    /// Wait for all spawned processes to complete successfully
    ///
    /// # Errors
    ///
    /// When anything goes wrong when waiting for a process, including non-zero exit codes.
    fn join(&mut self) -> Result<(), Error>;
}
