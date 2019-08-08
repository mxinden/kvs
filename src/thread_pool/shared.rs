use log::error;
use std::panic;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

use super::ThreadPool;
use crate::error::Result;

/// A shared thread pool implementation.
pub struct SharedQueueThreadPool {
    tx: Sender<Job>,
    rx: Arc<Mutex<Receiver<Job>>>,
    handles: Vec<thread::JoinHandle<()>>,
}

type Job = Box<FnOnce() + Send + 'static + std::panic::UnwindSafe>;

impl ThreadPool for SharedQueueThreadPool {
    /// Return new thread pool.
    fn new(threads: u32) -> Result<Self>
    where
        Self: Sized,
    {
        let (tx, rx) = channel::<Job>();
        let rx = Arc::new(Mutex::new(rx));

        let mut handles = vec![];

        for _ in 0..threads {
            let rx = rx.clone();

            handles.push(thread::spawn(move || {
                loop {
                    let rx = rx.lock().unwrap();

                    let job = rx.recv();

                    drop(rx);

                    match job {
                        Ok(job) => match panic::catch_unwind(job) {
                            Ok(()) => {}
                            Err(e) => error!("{:?}", e),
                        },
                        Err(e) => {
                            // Sender was dropped, thereby closing the thread.
                            return;
                        }
                    }
                }
            }));
        }

        Ok(SharedQueueThreadPool { tx, rx, handles })
    }

    /// Spawn the given job on the thread pool.
    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static + std::panic::UnwindSafe,
    {
        self.tx.send(Box::new(job));
    }
}

impl Drop for SharedQueueThreadPool {
    fn drop(&mut self) {
        let (tx, _rx) = channel::<Job>();

        let old_tx = std::mem::replace(&mut self.tx, tx);
        drop(old_tx);

        for handle in self.handles.drain(..) {
            handle.join().unwrap();
        }
    }
}
