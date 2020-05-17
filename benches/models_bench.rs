use criterion::{criterion_group, criterion_main, Criterion};

use glboot::core::window::Window;
use glboot::ogl::model::loaders;

pub fn obj_loader(c: &mut Criterion) {
    let mut w = Window::new("bench", (1, 1));
    w.load_gl();

    let path = format!("{}/assets/models/teapot.obj", env!("CARGO_MANIFEST_DIR"));
    c.bench_function("model", |b| b.iter(|| loaders::load_obj(&path).unwrap()));
}

criterion_group!(models, obj_loader);
criterion_main!(models);
