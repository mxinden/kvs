use crate::error::Result;

mod naive;
mod shared;
mod rayon;

pub use self::naive::NaiveThreadPool;
pub use self::shared::SharedQueueThreadPool;
pub use self::rayon::RayonThreadPool;

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


