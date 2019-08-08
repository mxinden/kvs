use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::sync::Mutex;

use kvs::thread_pool::*;
use kvs::Result;

use crossbeam_utils::sync::WaitGroup;

fn spawn_counter<P: ThreadPool>(pool: P) -> Result<()> {
    const TASK_NUM: usize = 20;
    const ADD_COUNT: usize = 1000;

    let counter = Arc::new(Mutex::new(0));

    for _ in 0..TASK_NUM {
        let counter = Arc::clone(&counter);
        pool.spawn(move || {
            for _ in 0..ADD_COUNT {
                *counter.lock().unwrap() += 1;
            }
        })
    }

    let start = std::time::Instant::now();

    while std::time::Instant::now() - start < std::time::Duration::from_secs(5) {
        if *counter.lock().unwrap() == TASK_NUM * ADD_COUNT {
            return Ok(());
        }
    }

    panic!("timeout");
}

fn spawn_panic_task<P: ThreadPool>() -> Result<()> {
    const TASK_NUM: usize = 1000;

    let pool = P::new(4)?;
    for _ in 0..TASK_NUM {
        pool.spawn(move || {
            // It suppresses flood of panic messages to the console.
            // You may find it useful to comment this out during development.
            panic_control::disable_hook_in_current_thread();

            panic!();
        })
    }

    spawn_counter(pool)
}

#[test]
fn naive_thread_pool_spawn_counter() -> Result<()> {
    let pool = NaiveThreadPool::new(4)?;
    spawn_counter(pool)
}

#[test]
fn shared_queue_thread_pool_spawn_counter() -> Result<()> {
    let pool = SharedQueueThreadPool::new(4)?;
    spawn_counter(pool)
}

#[test]
fn rayon_thread_pool_spawn_counter() -> Result<()> {
    let pool = RayonThreadPool::new(4)?;
    spawn_counter(pool)
}

#[test]
fn shared_queue_thread_pool_panic_task() -> Result<()> {
    spawn_panic_task::<SharedQueueThreadPool>()
}
