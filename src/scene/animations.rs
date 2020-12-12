use cgmath::{Quaternion, Vector3};
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
}

impl Animation {
    pub fn new(anim: gltf::Animation, buf: &[gltf::buffer::Data]) -> Self {
        Self {
            channels: anim.channels().map(|ch| Channel::new(ch, buf)).collect(),
        }
    }

    pub fn animate(&self, time: f32, nodes: &mut [super::Node]) {
        for channel in self.channels.iter() {
            let time = if time > 1.0 {
                time - time.trunc()
            } else {
                time
            };

            dbg!(time);

            let (prev_ind, prev) = channel
                .input
                .iter()
                .enumerate()
                .filter(|(_, &x)| x <= time)
                .max_by(|&x, &y| x.1.partial_cmp(y.1).unwrap())
                .unwrap();
            let (next_ind, next) = channel
                .input
                .iter()
                .enumerate()
                .filter(|(_, &x)| x >= time)
                .min_by(|&x, &y| x.1.partial_cmp(y.1).unwrap())
                .unwrap();

            match channel.path {
                gltf::animation::Property::Rotation => {
                    let prev_quat = channel.output.get_rotation(prev_ind);
                    let next_quat = channel.output.get_rotation(next_ind);

                    let interpolation = (time - prev) / (next - prev);

                    nodes[channel.target].rotation = prev_quat.slerp(next_quat, interpolation);
                    nodes[channel.target].update();
                }
                _ => unimplemented!(),
            }
        }
    }
}

impl Channel {
    fn new(channel: gltf::animation::Channel, buf: &[gltf::buffer::Data]) -> Self {
        let reader = channel.reader(|buffer| Some(&buf[buffer.index()]));

        let target = channel.target().node().index();
        let path = channel.target().property();
        let input = reader.read_inputs().unwrap().collect();
        let output = Output::from(reader.read_outputs().unwrap());

        Self {
            target,
            path,
            input,
            output,
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
