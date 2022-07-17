//! Simple command process pool
//!
//! This is a simple pool to manage limiting the number of spawned processes, and manage cleanup so
//! there are no zombie processes. To effectively manage cleanup, this needs to be dropped, so
//! panics while using this may result in zombie processes
use std::collections::VecDeque;
use std::io::{Error, ErrorKind, Result};
use std::process::{Child, Command, ExitStatus};

/// A pool to manage spawning a limited number of processses
///
/// This design is simple and will wait for the old scheduled process to complete before scheduling
/// a new one. If you schedule a long running process, then a bunch of short ones, it won't
/// schedule more short ones beyond the buffer until the long one has finished.
#[derive(Debug)]
pub struct Pool {
    procs: VecDeque<Child>,
    max_procs: usize,
}

fn parse_code(status: ExitStatus) -> Result<()> {
    match status.code() {
        Some(0) => Ok(()),
        Some(code) => Err(Error::new(
            ErrorKind::Other,
            format!("child process finished with nonzero exit code: {}", code),
        )),
        None => Err(Error::new(
            ErrorKind::Other,
            "child process was killed by a signal",
        )),
    }
}

/// internal function to wait for a process and error in the event that it doesn't complete
/// successfully (non-zero error code or otherwise)
fn wait_proc(proc: &mut Child) -> Result<()> {
    parse_code(proc.wait()?)
}

impl Pool {
    /// Create a new empty pool with a limited number of total processes
    ///
    /// Set max_procs to 0 to enable unbounded parallelism.
    pub fn new(max_procs: usize) -> Pool {
        Pool {
            procs: VecDeque::new(),
            max_procs,
        }
    }

    /// Spawn a new process with command and return a mutable reference to the process
    ///
    /// This command will block until it can schedule the process under the constraints. It can
    /// fail for any reason, including an earlier process failed, and never actually spawn the
    /// process in question. If it does successfully spawn the process, it will be recorded so that
    /// it will be cleaned up if the pool is dropped.
    pub fn spawn(&mut self, command: &mut Command) -> Result<&mut Child> {
        // check if early processes have finished and clean them up
        while let Some(proc) = self.procs.front_mut() {
            match proc.try_wait()? {
                Some(status) => {
                    parse_code(status)?;
                    self.procs.pop_front();
                }
                None => break,
            }
        }

        // next wait for oldest proc to finish if we're full
        if self.procs.len() == self.max_procs {
            if let Some(mut proc) = self.procs.pop_front() {
                wait_proc(&mut proc)?
            }
        };

        // now schedule new process
        self.procs.push_back(command.spawn()?);
        Ok(self.procs.back_mut().unwrap()) // just pushed
    }

    /// wait for all processes to finish
    ///
    /// Errors will terminate early and not wait for reamining processes to finish. To continue
    /// waiting for them anyway you can continue to call join until you get a success, this will
    /// indicate that there are no more running processes under management by the pool.
    pub fn join(&mut self) -> Result<()> {
        while let Some(mut proc) = self.procs.pop_front() {
            wait_proc(&mut proc)?
        }
        Ok(())
    }
}

impl Drop for Pool {
    fn drop(&mut self) {
        // kill any children left in self
        for proc in &mut self.procs {
            let _ = proc.kill();
        }
        // wait for them to be cleaned up
        for proc in &mut self.procs {
            let _ = wait_proc(proc);
        }
    }
}
