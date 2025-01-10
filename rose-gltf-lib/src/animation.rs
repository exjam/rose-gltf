use bytes::{BufMut, BytesMut};
use rose_file_lib::files::{zmo, ZMO};
use serde_json::json;

use gltf_json::{
    accessor, animation, buffer,
    validation::{Checked, USize64},
    Index, Node,
};

use crate::pad_align;

pub trait GetAnimationChannelNode {
    fn get(&self, root: &mut gltf_json::Root, channel: u32) -> Index<Node>;
}

pub fn load_animation(
    root: &mut gltf_json::Root,
    binary_data: &mut BytesMut,
    zmo: &ZMO,
    name: &str,
    channel_nodes: impl GetAnimationChannelNode,
) {
    let mut channels = Vec::new();
    let mut samplers = Vec::new();

    pad_align(binary_data);

    let keyframe_time_start = binary_data.len();
    let fps = zmo.fps as f32;
    for i in 0..zmo.frames {
        binary_data.put_f32_le(i as f32 / fps)
    }
    let keyframe_time_length = binary_data.len() - keyframe_time_start;

    let buffer_view_index = root.buffer_views.len() as u32;
    root.buffer_views.push(buffer::View {
        name: Some(format!("{}_KeyframeTimesBuferView", name)),
        buffer: Index::new(0),
        byte_length: USize64::from(keyframe_time_length),
        byte_offset: Some(USize64::from(keyframe_time_start)),
        byte_stride: None,
        extensions: Default::default(),
        extras: Default::default(),
        target: None,
    });

    let keyframe_time_accessor_index = Index::new(root.accessors.len() as u32);
    root.accessors.push(accessor::Accessor {
        name: Some(format!("{}_KeyframeTimesAccessor", name)),
        buffer_view: Some(Index::new(buffer_view_index)),
        byte_offset: Some(USize64(0)),
        count: USize64::from(zmo.frames as usize),
        component_type: Checked::Valid(accessor::GenericComponentType(
            accessor::ComponentType::F32,
        )),
        extensions: Default::default(),
        extras: Default::default(),
        type_: Checked::Valid(accessor::Type::Scalar),
        min: Some(json!([0.0])),
        max: Some(json!([(zmo.frames - 1) as f32 / fps])),
        normalized: false,
        sparse: None,
    });

    for (channel_id, channel) in zmo.channels.iter().enumerate() {
        if !matches!(
            channel.typ,
            zmo::ChannelType::Position | zmo::ChannelType::Rotation | zmo::ChannelType::Scale
        ) {
            continue;
        }

        let keyframe_data_start = binary_data.len();
        match &channel.frames {
            zmo::ChannelData::Position(positions) => {
                for position in positions.iter() {
                    binary_data.put_f32_le(position.x / 100.0);
                    binary_data.put_f32_le(position.z / 100.0);
                    binary_data.put_f32_le(-position.y / 100.0);
                }
            }
            zmo::ChannelData::Rotation(rotations) => {
                for rotation in rotations.iter() {
                    binary_data.put_f32_le(rotation.x);
                    binary_data.put_f32_le(rotation.z);
                    binary_data.put_f32_le(-rotation.y);
                    binary_data.put_f32_le(rotation.w);
                }
            }
            zmo::ChannelData::Scale(scales) => {
                for scale in scales.iter() {
                    binary_data.put_f32_le(*scale);
                    binary_data.put_f32_le(*scale);
                    binary_data.put_f32_le(*scale);
                }
            }
            _ => unreachable!(),
        };
        let keyframe_data_length = binary_data.len() - keyframe_data_start;

        let buffer_view_index = root.buffer_views.len() as u32;
        root.buffer_views.push(buffer::View {
            name: Some(format!("{}_Channel{}_DataBufferView", name, channel_id)),
            buffer: Index::new(0),
            byte_length: USize64::from(keyframe_data_length),
            byte_offset: Some(USize64::from(keyframe_data_start)),
            byte_stride: None,
            extensions: Default::default(),
            extras: Default::default(),
            target: None,
        });

        let keyframe_data_accessor_index = Index::new(root.accessors.len() as u32);
        root.accessors.push(accessor::Accessor {
            name: Some(format!("{}_Channel{}_DataAccessor", name, channel_id)),
            buffer_view: Some(Index::new(buffer_view_index)),
            byte_offset: Some(USize64(0)),
            count: USize64::from(zmo.frames as usize),
            component_type: Checked::Valid(accessor::GenericComponentType(
                accessor::ComponentType::F32,
            )),
            extensions: Default::default(),
            extras: Default::default(),
            type_: Checked::Valid(if matches!(channel.typ, zmo::ChannelType::Rotation) {
                accessor::Type::Vec4
            } else {
                accessor::Type::Vec3
            }),
            min: None,
            max: None,
            normalized: false,
            sparse: None,
        });

        let sampler_index = Index::new(samplers.len() as u32);
        samplers.push(animation::Sampler {
            input: keyframe_time_accessor_index,
            interpolation: Checked::Valid(animation::Interpolation::Linear),
            output: keyframe_data_accessor_index,
            extensions: Default::default(),
            extras: Default::default(),
        });

        channels.push(animation::Channel {
            sampler: sampler_index,
            target: animation::Target {
                node: channel_nodes.get(root, channel.index),
                path: Checked::Valid(match channel.typ {
                    zmo::ChannelType::Position => animation::Property::Translation,
                    zmo::ChannelType::Rotation => animation::Property::Rotation,
                    zmo::ChannelType::Scale => animation::Property::Scale,
                    _ => unreachable!(),
                }),
                extensions: Default::default(),
                extras: Default::default(),
            },
            extensions: Default::default(),
            extras: Default::default(),
        });
    }

    root.animations.push(animation::Animation {
        extensions: Default::default(),
        extras: Default::default(),
        channels,
        name: Some(name.to_string()),
        samplers,
    });
}
