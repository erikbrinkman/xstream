//! Limiting process pool
//!
//! This is a process pool to manage limiting the number of spawned processes, and manage cleanup
//! so there are no zombie processes. When trying to spawn more than the limit, the old process
//! will be waited on before spawning a new one. To effectively manage cleanup, this needs to be
//! dropped, so panics while using this may result in zombie processes.
use super::pool;
use super::pool::{Error, Pool};
use std::borrow::BorrowMut;
use std::collections::VecDeque;
use std::process::{Child, Command, Stdio};

// TODO implement a better limited pool that pipes to the next completed one
/// A pool to manage spawning a limited number of processses
///
/// This will wait for the oldest scheduled process to complete before scheduling a new one. If you
/// schedule a long running process, then a bunch of short ones, it won't schedule more short ones
/// beyond the buffer until the long one has finished.
#[derive(Debug)]
pub struct Limiting<C> {
    procs: VecDeque<Child>,
    max_procs: usize,
    command: C,
}

impl<C: BorrowMut<Command>> Limiting<C> {
    /// Create a new empty pool with a limited number of total processes
    ///
    /// Set `max_procs` to 0 to enable unbounded parallelism.
    pub fn new(mut command: C, max_procs: usize) -> Self {
        command.borrow_mut().stdin(Stdio::piped());
        Limiting {
            procs: VecDeque::with_capacity(max_procs),
            max_procs,
            command,
        }
    }
}

impl<C: BorrowMut<Command>> Pool for Limiting<C> {
    /// Spawn a new process with command and return a mutable reference to the process
    ///
    /// This command will block until it can schedule the process under the constraints. It can
    /// fail for any reason, including an earlier process failed, and never actually spawn the
    /// process in question. If it does successfully spawn the process, it will be recorded so that
    /// it will be cleaned up if the pool is dropped.
    fn get(&mut self) -> Result<&mut Child, Error> {
        // wait for the oldest process if we're bounded
        if self.max_procs != 0 && self.procs.len() == self.max_procs {
            pool::wait_proc(self.procs.pop_front().unwrap())?;
        };

        // now schedule new process
        let proc = self.command.borrow_mut().spawn().map_err(Error::Spawn)?;
        self.procs.push_back(proc);
        Ok(self.procs.back_mut().unwrap()) // just pushed
    }

    /// Wait for all processes to finish
    ///
    /// Errors will terminate early and not wait for reamining processes to finish. To continue
    /// waiting for them anyway you can continue to call join until you get a success, this will
    /// indicate that there are no more running processes under management by the pool.
    fn join(&mut self) -> Result<(), Error> {
        // NOTE we do this instead of drain so that errors don't drop the rest of our processes
        // creating zombies
        while let Some(proc) = self.procs.pop_back() {
            pool::wait_proc(proc)?;
        }
        Ok(())
    }
}

impl<C> Drop for Limiting<C> {
    fn drop(&mut self) {
        // kill any children left in self
        for proc in &mut self.procs {
            let _ = proc.kill();
        }
        // wait for them to be cleaned up
        for proc in &mut self.procs {
            let _ = proc.wait();
        }
    }
}
