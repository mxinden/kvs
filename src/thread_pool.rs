use crate::error::Result;
use std::thread;

/// An abstraction of a thread pool.
pub trait ThreadPool {
    /// Return new thread pool.
    fn new(threads: u32) -> Result<Self>
    where
        Self: Sized;
    /// Spawn the given job on the thread pool.
    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static;
}

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

/// A shared thread pool implementation.
pub struct SharedQueueThreadPool {}

impl ThreadPool for SharedQueueThreadPool {
    /// Return new thread pool.
    fn new(threads: u32) -> Result<Self>
    where
        Self: Sized,
    {
        unimplemented!();
    }

    /// Spawn the given job on the thread pool.
    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        unimplemented!();
    }
}

/// Adapter for the rayon thread pool crate.
pub struct RayonThreadPool {}

impl ThreadPool for RayonThreadPool {
    /// Return new thread pool.
    fn new(threads: u32) -> Result<Self>
    where
        Self: Sized,
    {
        unimplemented!();
    }

    /// Spawn the given job on the thread pool.
    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        unimplemented!();
    }
}
