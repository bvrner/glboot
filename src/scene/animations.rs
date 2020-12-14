use cgmath::{Quaternion, Vector3, VectorSpace};
use gltf::animation::util::ReadOutputs;

#[derive(Debug, Clone)]
pub struct Animation {
    channels: Vec<Channel>,
}

#[derive(Debug, Clone)]
struct Channel {
    target: usize,
    path: gltf::animation::Property,
    input: Vec<f32>,
    output: Output,
    interpolation: gltf::animation::Interpolation,

    time_data: TimeData,
}

#[derive(Debug, Copy, Clone, Default)]
struct TimeData {
    time_accum: f32,
    frame_dur: f32,

    curr_time: f32,
    old_time: f32,

    end_time: f32,
    start_time: f32,

    prev_index: usize,
    next_index: usize,
    end_index: usize,
    interp: f32,
}

pub fn clamp(val: usize, min: usize, max: usize) -> usize {
    assert!(min <= max);
    let mut x = val;
    if x < min {
        x = min;
    }
    if x > max {
        x = max;
    }
    x
}

impl TimeData {
    fn update(&mut self, time: f32, inputs: &[f32]) {
        self.time_accum = time;
        self.curr_time = (self.time_accum % self.end_time).max(self.start_time);

        if self.old_time > self.curr_time {
            self.prev_index = 0;
        }

        self.old_time = self.curr_time;

        // Find next keyframe: min{ t of input | t > prevKey }
        let mut next_key = 0;
        for i in self.prev_index..inputs.len() {
            if self.curr_time <= inputs[i] {
                next_key = clamp(i, 1, inputs.len() - 1);
                break;
            }
        }

        self.prev_index = clamp(next_key - 1, 0, next_key);

        let delta = inputs[next_key] - inputs[self.prev_index];

        // Normalize t: [t0, t1] -> [0, 1]
        self.interp = (self.curr_time - inputs[self.prev_index]) / delta;
        self.next_index = next_key;
        // self.curr_time += time;
    }
}

#[derive(Debug, Clone)]
enum Output {
    Translation(Vec<Vector3<f32>>),
    Rotation(Vec<Quaternion<f32>>),
    Scale(Vec<Vector3<f32>>),
    // Morph,
}

impl Output {
    #[track_caller]
    fn get_rotation(&self, ind: usize) -> Quaternion<f32> {
        match self {
            Output::Rotation(q) => q[ind],
            _ => panic!(),
        }
    }

    #[track_caller]
    fn get_translation(&self, ind: usize) -> Vector3<f32> {
        match self {
            Output::Translation(t) => t[ind],
            _ => panic!(),
        }
    }
}

impl Animation {
    pub fn new(anim: gltf::Animation, buf: &[gltf::buffer::Data]) -> Self {
        Self {
            channels: anim.channels().map(|ch| Channel::new(ch, buf)).collect(),
        }
    }

    pub fn animate(&mut self, time_: f32, nodes: &mut [super::Node]) {
        for channel in self.channels.iter_mut() {
            channel.time_data.update(time_, &channel.input);
            let time = &channel.time_data;

            match channel.path {
                gltf::animation::Property::Rotation => {
                    let prev_quat = channel.output.get_rotation(time.prev_index);
                    let next_quat = channel.output.get_rotation(time.next_index);
                    // dbg!(*time, interpolation, &channel.input);
                    dbg!(*time);
                    // dbg!(interpolation);

                    nodes[channel.target].rotation = prev_quat.slerp(next_quat, time.interp);
                    // dbg!(&nodes[channel.target].rotation);
                }
                gltf::animation::Property::Translation => {
                    let prev_trans = channel.output.get_translation(time.prev_index);
                    let next_trans = channel.output.get_translation(time.next_index);

                    nodes[channel.target].translation = prev_trans.lerp(next_trans, time.interp);
                }
                _ => {}
            }
            nodes[channel.target].update();
        }
    }
}

impl Channel {
    fn new(channel: gltf::animation::Channel, buf: &[gltf::buffer::Data]) -> Self {
        let reader = channel.reader(|buffer| Some(&buf[buffer.index()]));

        let target = channel.target().node().index();
        let path = channel.target().property();
        let input: Vec<f32> = reader.read_inputs().unwrap().collect();
        let output = Output::from(reader.read_outputs().unwrap());
        let time_data = TimeData {
            interp: 0.0,
            time_accum: 0.0,
            frame_dur: input[1] - input[0],
            curr_time: input[0],
            old_time: 0.0,
            end_time: input[input.len() - 1],
            start_time: input[0],
            prev_index: 0,
            next_index: 1,
            end_index: input.len() - 1,
        };

        Self {
            target,
            path,
            input,
            output,
            time_data,
            interpolation: channel.sampler().interpolation(),
        }
    }
}

impl<'a> From<ReadOutputs<'a>> for Output {
    fn from(out: ReadOutputs) -> Self {
        match out {
            ReadOutputs::Translations(t) => Self::Translation(t.map(Into::into).collect()),
            ReadOutputs::Rotations(r) => {
                let quat = r
                    .into_f32()
                    .map(|r| Quaternion::new(r[3], r[0], r[1], r[2]));
                Self::Rotation(quat.collect())
            }
            ReadOutputs::Scales(s) => Self::Scale(s.map(Into::into).collect()),
            ReadOutputs::MorphTargetWeights(_) => unimplemented!(),
        }
    }
}
