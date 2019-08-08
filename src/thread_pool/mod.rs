use crate::error::Result;

mod naive;
mod shared;

pub use self::naive::NaiveThreadPool;
pub use self::shared::SharedQueueThreadPool;

/// An abstraction of a thread pool.
pub trait ThreadPool {
    /// Return new thread pool.
    fn new(threads: u32) -> Result<Self>
    where
        Self: Sized;
    /// Spawn the given job on the thread pool.
    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static + std::panic::UnwindSafe;
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
