use criterion::{criterion_group, criterion_main, Criterion};

use glboot::core::window::Window;
use glboot::ogl::model::{FullVertex, loaders};
use glboot::ogl::model::mesh::Model;

use std::time::Duration;

pub fn obj_loader(c: &mut Criterion) {
    let mut w = Window::hidden("bench", (1, 1));
    w.load_gl();

    let tea_path = format!("{}/assets/models/teapot.obj", env!("CARGO_MANIFEST_DIR"));
    let mut group = c.benchmark_group("Models");

    group.measurement_time(Duration::from_secs(30));
    group.bench_function("teapot", |b| {
        b.iter(|| { let model: Model<FullVertex> = loaders::load_obj(&tea_path).unwrap(); })
    });

    group.finish();
}

criterion_group!(models, obj_loader);
criterion_main!(models);
