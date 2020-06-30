use criterion::{criterion_group, criterion_main, Criterion};

use glboot::core::window::Window;
use glboot::ogl::model::mesh::Model;
use glboot::ogl::model::{loaders, FullVertex};

use std::time::Duration;

pub fn gltf_loader(c: &mut Criterion) {
    let mut w = Window::hidden();
    w.load_gl();

    let m_path = format!(
        "{}/assets/models/matilda/scene.gltf",
        env!("CARGO_MANIFEST_DIR")
    );
    let d_path = format!(
        "{}/assets/models/simpler_dragon.glb",
        env!("CARGO_MANIFEST_DIR")
    );
    let mut group = c.benchmark_group("Models");

    group.measurement_time(Duration::from_secs(30));
    group.bench_function("matilda", |b| {
        b.iter(|| {
            let model: Model<FullVertex> = loaders::load_gltf(&m_path).unwrap();
        })
    });
    group.bench_function("binary dragon", |b| {
        b.iter(|| {
            let model: Model<FullVertex> = loaders::load_gltf(&d_path).unwrap();
        })
    });

    group.finish();
}

// pub fn obj_loader(c: &mut Criterion) {
//     let mut w = Window::hidden();
//     w.load_gl();

//     let tea_path = format!("{}/assets/models/teapot.obj", env!("CARGO_MANIFEST_DIR"));
//     let mut group = c.benchmark_group("Models");

//     group.measurement_time(Duration::from_secs(30));
//     group.bench_function("teapot", |b| {
//         b.iter(|| {
//             let model: Model<FullVertex> = loaders::load_obj(&tea_path).unwrap();
//         })
//     });

//     group.finish();
// }

criterion_group!(models, gltf_loader);
criterion_main!(models);
