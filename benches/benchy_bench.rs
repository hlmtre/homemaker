use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hm::{
  config::{deserialize_file, Config, ManagedObject},
  get_task_batches,
};
use std::collections::HashMap;

fn criterion_benchmark(c: &mut Criterion) {
  let a: Config = deserialize_file("./src/config.toml").unwrap();
  let nodes: HashMap<String, ManagedObject> = Config::as_managed_objects(a);
  c.bench_function("get_task_bathces", |b| {
    b.iter(|| get_task_batches(nodes.clone()))
  });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
