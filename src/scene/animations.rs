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
    start: usize,  // first frame index
    end: usize,    // last frame inde
    end_time: f32, // last time frame

    curr_time: f32,

    curr_frame: usize, // current frame index
    next_frame: usize, // next frame index
}

impl TimeData {
    fn update(&mut self, time: f32) {
        self.curr_time += time;

        if self.curr_time > self.end_time {
            self.curr_time -= self.end_time;

            self.curr_frame = self.end;
            self.next_frame = self.start;
        }
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
            let time = &mut channel.time_data;
            time.update(time_);

            let prev_time = channel.input[time.curr_frame];
            let next_time = channel.input[time.next_frame];
            let interpolation = (time.curr_time - prev_time) / (next_time - prev_time);

            match channel.path {
                gltf::animation::Property::Rotation => {
                    let prev_quat = channel.output.get_rotation(time.curr_frame);
                    let next_quat = channel.output.get_rotation(time.next_frame);

                    nodes[channel.target].rotation = prev_quat.slerp(next_quat, interpolation);
                }
                gltf::animation::Property::Translation => {
                    let prev_trans = channel.output.get_translation(time.curr_frame);
                    let next_trans = channel.output.get_translation(time.next_frame);
                    dbg!(*time);

                    nodes[channel.target].translation = prev_trans.lerp(next_trans, interpolation);
                }
                _ => unimplemented!(),
            }
            nodes[channel.target].update();

            if time.curr_time > channel.input[time.next_frame] {
                time.curr_frame = time.next_frame;
                time.next_frame += 1;

                if time.next_frame > time.end {
                    time.next_frame = time.start;
                }
            }
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
        let mut time_data = TimeData::default();

        time_data.end = input.len() - 1;
        time_data.end_time = *input.last().unwrap();
        time_data.next_frame = 1;

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
