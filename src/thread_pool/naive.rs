use std::thread;

use super::ThreadPool;
use crate::error::Result;

/// A naive thread pool implementation.
pub struct NaiveThreadPool {}

impl ThreadPool for NaiveThreadPool {
    /// Return new thread pool.
    fn new(threads: u32) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(NaiveThreadPool{})
    }

    /// Spawn the given job on the thread pool.
    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        println!("spawning a new thread");
        thread::spawn(job);
    }
}
