use crate::slerp;
use cgmath::{prelude::*, Quaternion, Vector3, VectorSpace};
use gltf::animation::{util::ReadOutputs, Channel as gltfChannel, Interpolation, Property};

#[derive(Debug, Clone)]
pub struct Animation {
    rotations: Vec<Channel<Quaternion<f32>>>,
    translations: Vec<Channel<Vector3<f32>>>,
    scales: Vec<Channel<Vector3<f32>>>,

    pub name: String,
}

#[derive(Debug, Clone)]
struct Channel<T> {
    target: usize,
    path: Property,
    input: Vec<f32>,
    output: Vec<T>,
    interpolation: Interpolation,

    frame: FrameData,
}

#[derive(Debug, Copy, Clone, Default)]
struct FrameData {
    time_accum: f32, // total time since the animation started
    curr_time: f32,  // time relative to the beginning and end of the animation

    end_time: f32,   // last frame time
    start_time: f32, // first frame time

    prev_index: usize, // index of the last frame
    next_index: usize, // index of the next frame

    interp: f32, // interpolation factor of the current frame
}

impl FrameData {
    fn update(&mut self, time: f32, inputs: &[f32]) {
        self.time_accum += time;
        self.curr_time = (self.time_accum % self.end_time).max(self.start_time);

        let mut index = 0;
        for (i, win) in inputs.windows(2).enumerate() {
            let previous = win[0];
            let next = win[1];
            if self.curr_time >= previous && self.curr_time < next {
                index = i;
                break;
            }
        }

        let previous_time = inputs[index];
        let next_time = inputs[index + 1];
        let delta = next_time - previous_time;
        let from_start = self.curr_time - previous_time;

        self.prev_index = index;
        self.next_index = index + 1;
        self.interp = from_start / delta
    }
}

impl Animation {
    pub fn new(anim: &gltf::Animation, buf: &[gltf::buffer::Data]) -> Self {
        Self {
            rotations: anim
                .channels()
                .filter(|ch| ch.target().property() == Property::Rotation)
                .map(|ch| -> Channel<Quaternion<f32>> {
                    Channel::<Quaternion<f32>>::new_rotation(ch, buf)
                })
                .collect(),
            translations: anim
                .channels()
                .filter(|ch| ch.target().property() == Property::Translation)
                .map(|ch| Channel::<Vector3<f32>>::new_trans(ch, buf))
                .collect(),
            scales: anim
                .channels()
                .filter(|ch| ch.target().property() == Property::Scale)
                .map(|ch| Channel::<Vector3<f32>>::new_scale(ch, buf))
                .collect(),
            name: anim.name().map_or(anim.index().to_string(), String::from),
        }
    }

    pub fn animate(&mut self, time: f32, nodes: &mut [super::Node]) {
        self.translations
            .iter_mut()
            .map(|ch| ch.animate(time))
            .for_each(|(index, t)| {
                nodes[index].translation = t;
                nodes[index].update()
            });
        self.scales
            .iter_mut()
            .map(|ch| ch.animate(time))
            .for_each(|(index, t)| {
                nodes[index].scale = t;
                nodes[index].update()
            });
        self.rotations
            .iter_mut()
            .map(|ch| ch.animate(time))
            .for_each(|(index, t)| {
                nodes[index].rotation = t;
                nodes[index].update()
            });
    }
}

impl<T: Interpolate + Copy> Channel<T> {
    fn animate(&mut self, t: f32) -> (usize, T) {
        self.frame.update(t, &self.input);

        let i = self.frame.prev_index;
        let j = self.frame.next_index;

        match self.interpolation {
            Interpolation::Linear => {
                let transform = self.output[i].linear(self.output[j], self.frame.interp);
                (self.target, transform)
            }
            Interpolation::CubicSpline => {
                let previous_values = [
                    self.output[i * 3],
                    self.output[i * 3 + 1],
                    self.output[i * 3 + 2],
                ];
                let next_values = [
                    self.output[i * 3 + 3],
                    self.output[i * 3 + 4],
                    self.output[i * 3 + 5],
                ];
                let t = Interpolate::cubic(
                    previous_values,
                    self.input[i],
                    next_values,
                    self.input[j],
                    self.frame.interp,
                );

                (self.target, t)
            }
            Interpolation::Step => (self.target, self.output[i]),
        }
    }
}

impl<T> Channel<T> {
    fn new_rotation(ch: gltfChannel, buf: &[gltf::buffer::Data]) -> Channel<Quaternion<f32>> {
        let reader = ch.reader(|buffer| Some(&buf[buffer.index()]));
        let target = ch.target().node().index();
        let path = ch.target().property();

        let input: Vec<f32> = reader.read_inputs().unwrap().collect();
        let output: Vec<Quaternion<f32>> =
            reader.read_outputs().map_or(vec![], |output| match output {
                ReadOutputs::Rotations(r) => r
                    .into_f32()
                    .map(|arr| Quaternion::new(arr[3], arr[0], arr[1], arr[2]))
                    .collect(),
                _ => vec![],
            });
        let frame = FrameData {
            interp: 0.0,
            time_accum: 0.0,
            curr_time: input[0],
            end_time: input[input.len() - 1],
            start_time: input[0],
            prev_index: 0,
            next_index: 1,
        };

        Channel {
            target,
            path,
            input,
            output,
            interpolation: ch.sampler().interpolation(),
            frame,
        }
    }
    fn new_scale(ch: gltfChannel, buf: &[gltf::buffer::Data]) -> Channel<Vector3<f32>> {
        let reader = ch.reader(|buffer| Some(&buf[buffer.index()]));
        let target = ch.target().node().index();
        let path = ch.target().property();

        let input: Vec<f32> = reader.read_inputs().unwrap().collect();
        let output: Vec<Vector3<f32>> =
            reader.read_outputs().map_or(vec![], |output| match output {
                ReadOutputs::Scales(s) => s.map(Vector3::from).collect(),
                _ => vec![],
            });
        let frame = FrameData {
            interp: 0.0,
            time_accum: 0.0,
            curr_time: input[0],
            end_time: input[input.len() - 1],
            start_time: input[0],
            prev_index: 0,
            next_index: 1,
        };

        Channel {
            target,
            path,
            input,
            output,
            interpolation: ch.sampler().interpolation(),
            frame,
        }
    }
    fn new_trans(ch: gltfChannel, buf: &[gltf::buffer::Data]) -> Channel<Vector3<f32>> {
        let reader = ch.reader(|buffer| Some(&buf[buffer.index()]));
        let target = ch.target().node().index();
        let path = ch.target().property();

        let input: Vec<f32> = reader.read_inputs().unwrap().collect();
        let output: Vec<Vector3<f32>> =
            reader.read_outputs().map_or(vec![], |output| match output {
                ReadOutputs::Translations(ts) => ts.map(Vector3::from).collect(),
                _ => vec![],
            });
        let frame = FrameData {
            interp: 0.0,
            time_accum: 0.0,
            curr_time: input[0],
            end_time: input[input.len() - 1],
            start_time: input[0],
            prev_index: 0,
            next_index: 1,
        };

        Channel {
            target,
            path,
            input,
            output,
            interpolation: ch.sampler().interpolation(),
            frame,
        }
    }
}

trait Interpolate: Sized {
    fn linear(&self, other: Self, t: f32) -> Self;
    fn cubic(source: [Self; 3], stime: f32, target: [Self; 3], ttime: f32, t: f32) -> Self;
}

impl Interpolate for Vector3<f32> {
    fn linear(&self, other: Self, t: f32) -> Self {
        self.lerp(other, t)
    }

    fn cubic(source: [Self; 3], stime: f32, target: [Self; 3], ttime: f32, t: f32) -> Self {
        let p0 = source[1];
        let m0 = (ttime - stime) * source[2];
        let p1 = target[1];
        let m1 = (ttime - stime) * target[0];

        (2.0 * t * t * t - 3.0 * t * t + 1.0) * p0
            + (t * t * t - 2.0 * t * t + t) * m0
            + (-2.0 * t * t * t + 3.0 * t * t) * p1
            + (t * t * t - t * t) * m1
    }
}

impl Interpolate for Quaternion<f32> {
    fn linear(&self, other: Self, t: f32) -> Self {
        slerp(*self, other, t)
    }

    fn cubic(source: [Self; 3], stime: f32, target: [Self; 3], ttime: f32, t: f32) -> Self {
        let p0 = source[1];
        let m0 = (ttime - stime) * source[2];
        let p1 = target[1];
        let m1 = (ttime - stime) * target[0];

        let ret = (2.0 * t * t * t - 3.0 * t * t + 1.0) * p0
            + (t * t * t - 2.0 * t * t + t) * m0
            + (-2.0 * t * t * t + 3.0 * t * t) * p1
            + (t * t * t - t * t) * m1;
        ret.normalize()
    }
}
