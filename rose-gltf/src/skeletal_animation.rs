use bytes::{BufMut, BytesMut};
use glam::{Mat4, Quat, Vec3};
use roselib::files::{zmo, ZMD, ZMO};
use serde_json::json;

use gltf_json::{
    accessor, animation, buffer,
    scene::UnitQuaternion,
    validation::{Checked, USize64},
    Index, Node, Skin,
};

use crate::pad_align;

fn transform_children(zmd: &ZMD, bone_transforms: &mut Vec<Mat4>, bone_index: usize) {
    for (child_id, child_bone) in zmd.bones.iter().enumerate() {
        if child_id == bone_index || child_bone.parent as usize != bone_index {
            continue;
        }

        bone_transforms[child_id] = bone_transforms[bone_index] * bone_transforms[child_id];
        transform_children(zmd, bone_transforms, child_id);
    }
}

pub fn load_skeleton(
    root: &mut gltf_json::Root,
    binary_data: &mut BytesMut,
    name: &str,
    zmd: &ZMD,
) -> Index<Skin> {
    let bone_node_index_start = root.nodes.len();
    let mut joints = Vec::new();
    let mut bind_pose = Vec::new();

    pad_align(binary_data);

    // Add root node to scene
    root.scenes[0]
        .nodes
        .push(Index::new(root.nodes.len() as u32));

    // Create nodes for each bone
    for i in 0..zmd.bones.len() {
        let bone = &zmd.bones[i];
        let translation = Vec3::new(bone.position.x, bone.position.z, -bone.position.y) / 100.0;
        let rotation = Quat::from_xyzw(
            bone.rotation.x,
            bone.rotation.z,
            -bone.rotation.y,
            bone.rotation.w,
        )
        .normalize();

        root.nodes.push(Node {
            name: Some(format!("{}_Bone_{}", name, i)),
            camera: None,
            children: None,
            extensions: Default::default(),
            extras: Default::default(),
            matrix: None,
            mesh: None,
            rotation: Some(UnitQuaternion([
                rotation.x, rotation.y, rotation.z, rotation.w,
            ])),
            scale: None,
            translation: Some([translation.x, translation.y, translation.z]),
            skin: None,
            weights: None,
        });

        joints.push(Index::new(bone_node_index_start as u32 + i as u32));
        bind_pose.push(glam::Mat4::from_rotation_translation(rotation, translation));
    }

    // Assign parents
    for i in 0..zmd.bones.len() {
        let parent_bone_index = zmd.bones[i].parent as usize;
        if parent_bone_index == i {
            continue;
        }

        let parent = &mut root.nodes[bone_node_index_start + parent_bone_index];
        if let Some(children) = parent.children.as_mut() {
            children.push(Index::new(bone_node_index_start as u32 + i as u32));
        } else {
            parent.children = Some(vec![Index::new(bone_node_index_start as u32 + i as u32)]);
        }
    }

    // Calculate inverse bind pose
    transform_children(zmd, &mut bind_pose, 0);
    let inverse_bind_pose: Vec<Mat4> = bind_pose.iter().map(|x| x.inverse()).collect();

    let skeleton_data_start = binary_data.len();
    for mtx in inverse_bind_pose.iter() {
        for f in mtx.to_cols_array() {
            binary_data.put_f32_le(f);
        }
    }
    let skeleton_data_length = binary_data.len() - skeleton_data_start;

    let buffer_view_index = root.buffer_views.len() as u32;
    root.buffer_views.push(buffer::View {
        name: Some(format!("{}_SkeletonBufferView", name)),
        buffer: Index::new(0),
        byte_length: USize64::from(skeleton_data_length),
        byte_offset: Some(USize64::from(skeleton_data_start)),
        byte_stride: None,
        extensions: Default::default(),
        extras: Default::default(),
        target: None,
    });

    let accessor_index = root.accessors.len() as u32;
    root.accessors.push(accessor::Accessor {
        name: Some(format!("{}_SkeletonAccessor", name)),
        buffer_view: Some(Index::new(buffer_view_index)),
        byte_offset: Some(USize64(0)),
        count: USize64::from(inverse_bind_pose.len()),
        component_type: Checked::Valid(accessor::GenericComponentType(
            accessor::ComponentType::F32,
        )),
        extensions: Default::default(),
        extras: Default::default(),
        type_: Checked::Valid(accessor::Type::Mat4),
        min: None,
        max: None,
        normalized: false,
        sparse: None,
    });

    let skin_index = root.skins.len() as u32;
    root.skins.push(Skin {
        name: Some(name.to_string()),
        extensions: Default::default(),
        extras: Default::default(),
        inverse_bind_matrices: Some(Index::new(accessor_index)),
        skeleton: Some(joints[0]),
        joints,
    });
    Index::new(skin_index)
}

pub fn load_skeletal_animation(
    root: &mut gltf_json::Root,
    binary_data: &mut BytesMut,
    name: &str,
    skin_index: Index<Skin>,
    zmo: &ZMO,
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
                node: root.get(skin_index).unwrap().joints[channel.index as usize],
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
