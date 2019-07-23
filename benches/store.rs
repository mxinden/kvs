#[macro_use]
extern crate criterion;

use kvs::{KvStore, KvsEngine, Result};
use tempfile::TempDir;

use criterion::black_box;
use criterion::Criterion;

fn kv_store_benchmark(c: &mut Criterion) {
    c.bench_function("KvStore", |b| {
        b.iter_batched_ref(
            || {
                let temp_dir =
                    TempDir::new().expect("unable to create temporary working directory");
                KvStore::open(temp_dir.path()).unwrap()
            },
            |ref mut store| {
                for i in 0..1000 {
                    store
                        .set(format!("key-{}", i), format!("value-{}", i))
                        .unwrap();
                }
                for i in 0..1000 {
                    store.get(format!("key-{}", i)).unwrap();
                }
            },
            criterion::BatchSize::SmallInput,
        )
    });

    c.bench_function("SledKvsEngine", |b| {
        b.iter_batched_ref(
            || {
                let temp_dir =
                    TempDir::new().expect("unable to create temporary working directory");
                kvs::sled::SledKvsEngine::open(temp_dir.path()).unwrap()
            },
            |ref mut store| {
                for i in 0..1000 {
                    store
                        .set(format!("key-{}", i), format!("value-{}", i))
                        .unwrap();
                }
                for i in 0..1000 {
                    store.get(format!("key-{}", i)).unwrap();
                }
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

criterion_group!(benches, kv_store_benchmark);
criterion_main!(benches);
