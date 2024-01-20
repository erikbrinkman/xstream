//! Rotating command process pool
//!
//! This pool will spawn up to a set number of processes, and then start returning old processes in
//! a round robin fashion.  To effectively manage cleanup, this needs to be dropped, so panics
//! while using this may result in zombie processes.
use super::pool;
use super::pool::{Error, Pool};
use std::borrow::BorrowMut;
use std::process::{Child, Command, Stdio};

/// A pool to manage spawning a limited number of processses
///
/// This pool will return new processes up to the limit, and then start returning old processes in
/// a round-robin order. This type of pool is more effective if the process handles each task by
/// delimiters as well, allowing for better utilization of resources for embarassingly parallel
/// tasks.
#[derive(Debug)]
pub struct Rotating<C> {
    procs: Vec<Child>,
    max_procs: usize,
    command: C,
    ind: usize,
}

impl<C: BorrowMut<Command>> Rotating<C> {
    /// Create a new empty pool with a limited number of total processes
    ///
    /// Set `max_procs` to 0 to enable unbounded parallelism.
    pub fn new(mut command: C, max_procs: usize) -> Self {
        command.borrow_mut().stdin(Stdio::piped());
        Self {
            procs: Vec::with_capacity(max_procs),
            max_procs,
            command,
            ind: 0,
        }
    }

    /// Spawn a new process
    fn spawn(&mut self) -> Result<Child, Error> {
        self.command.borrow_mut().spawn().map_err(Error::Spawn)
    }
}

impl<C: BorrowMut<Command>> Pool for Rotating<C> {
    /// Get a process from the pool
    ///
    /// If fewer than `max_procs` have been spawned, this will spawn a new process, otherwise it
    /// will return one that was already spawned.
    fn get(&mut self) -> Result<&mut Child, Error> {
        if self.max_procs == 0 {
            let proc = self.spawn()?;
            self.procs.push(proc);
            Ok(self.procs.last_mut().unwrap())
        } else {
            if self.procs.len() < self.max_procs {
                let proc = self.spawn()?;
                self.procs.push(proc);
            }
            let child = &mut self.procs[self.ind];
            self.ind += 1;
            self.ind %= self.max_procs;
            Ok(child)
        }
    }

    /// Wait for all processes to finish successfully
    ///
    /// Errors will terminate early and not wait for reamining processes to finish. To continue
    /// waiting for them anyway you can continue to call join until you get a success, this will
    /// indicate that there are no more running processes under management by the pool.
    fn join(&mut self) -> Result<(), Error> {
        // NOTE we do this instead of drain so that errors don't drop the rest of our processes
        // creating zombies
        while let Some(proc) = self.procs.pop() {
            pool::wait_proc(proc)?;
        }
        Ok(())
    }
}

impl<C> Drop for Rotating<C> {
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
