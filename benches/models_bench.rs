use criterion::{criterion_group, criterion_main, Criterion};

use glboot::core::window::Window;
use glboot::ogl::model::loaders;

use std::time::Duration;

pub fn obj_loader(c: &mut Criterion) {
    let mut w = Window::new("bench", (1, 1));
    w.load_gl();

    let path = format!("{}/assets/models/teapot.obj", env!("CARGO_MANIFEST_DIR"));
    let mut group = c.benchmark_group("Model");
    group.measurement_time(Duration::from_secs(10));
    group.bench_function("obj", |b| b.iter(|| loaders::load_obj(&path).unwrap()));
    group.finish();
}

criterion_group!(models, obj_loader);
criterion_main!(models);
