use bytes::{BufMut, BytesMut};
use glam::{Mat4, Quat, Vec3};
use gltf::animation::Interpolation;
use gltf_json as json;
use json::{scene::UnitQuaternion, validation::Checked::Valid};
use roselib::{
    files::{
        zmo::{ChannelData, ChannelType},
        ZMD, ZMO, ZMS,
    },
    io::RoseFile,
};
use serde_json::json;
use std::{borrow::Cow, collections::HashMap, path::PathBuf};

fn load_mesh(root: &mut json::Root, binary_data: &mut BytesMut, name: &str, zms: &ZMS) -> u32 {
    let mut attributes_map = HashMap::new();
    let mut vertex_data_stride = 0;
    let vertex_count = zms.vertices.len();
    let vertex_data_buffer_view_index = root.buffer_views.len() as u32;
    let index_data_buffer_view_index = vertex_data_buffer_view_index + 1;

    if zms.positions_enabled() {
        let mut min_pos = zms.vertices[0].position;
        let mut max_pos = zms.vertices[0].position;
        for vertex in zms.vertices.iter() {
            min_pos.x = min_pos.x.min(vertex.position.x);
            min_pos.y = min_pos.y.min(vertex.position.y);
            min_pos.z = min_pos.z.min(vertex.position.z);

            max_pos.x = max_pos.x.max(vertex.position.x);
            max_pos.y = max_pos.y.max(vertex.position.y);
            max_pos.z = max_pos.z.max(vertex.position.z);
        }

        let accessor_index = root.accessors.len() as u32;
        root.accessors.push(json::Accessor {
            name: Some(format!("{}_Position", name)),
            buffer_view: Some(json::Index::new(vertex_data_buffer_view_index)),
            byte_offset: vertex_data_stride,
            count: vertex_count as u32,
            component_type: Valid(json::accessor::GenericComponentType(
                json::accessor::ComponentType::F32,
            )),
            extensions: Default::default(),
            extras: Default::default(),
            type_: Valid(json::accessor::Type::Vec3),
            min: Some(json!(vec![min_pos.x, min_pos.z, -min_pos.y])),
            max: Some(json!(vec![max_pos.x, max_pos.z, -max_pos.y])),
            normalized: false,
            sparse: None,
        });
        attributes_map.insert(
            Valid(json::mesh::Semantic::Positions),
            json::Index::new(accessor_index),
        );
        vertex_data_stride += 4 * 3;
    }

    if zms.normals_enabled() {
        let accessor_index = root.accessors.len() as u32;
        root.accessors.push(json::Accessor {
            name: Some(format!("{}_Normal", name)),
            buffer_view: Some(json::Index::new(vertex_data_buffer_view_index)),
            byte_offset: vertex_data_stride,
            count: vertex_count as u32,
            component_type: Valid(json::accessor::GenericComponentType(
                json::accessor::ComponentType::F32,
            )),
            extensions: Default::default(),
            extras: Default::default(),
            type_: Valid(json::accessor::Type::Vec3),
            min: None,
            max: None,
            normalized: false,
            sparse: None,
        });
        attributes_map.insert(
            Valid(json::mesh::Semantic::Normals),
            json::Index::new(accessor_index),
        );
        vertex_data_stride += 4 * 3;
    }

    if zms.tangents_enabled() {
        let accessor_index = root.accessors.len() as u32;
        root.accessors.push(json::Accessor {
            name: Some(format!("{}_Tangent", name)),
            buffer_view: Some(json::Index::new(vertex_data_buffer_view_index)),
            byte_offset: vertex_data_stride,
            count: vertex_count as u32,
            component_type: Valid(json::accessor::GenericComponentType(
                json::accessor::ComponentType::F32,
            )),
            extensions: Default::default(),
            extras: Default::default(),
            type_: Valid(json::accessor::Type::Vec3),
            min: None,
            max: None,
            normalized: false,
            sparse: None,
        });
        attributes_map.insert(
            Valid(json::mesh::Semantic::Tangents),
            json::Index::new(accessor_index),
        );
        vertex_data_stride += 4 * 3;
    }

    if zms.colors_enabled() {
        let accessor_index = root.accessors.len() as u32;
        root.accessors.push(json::Accessor {
            name: Some(format!("{}_Color", name)),
            buffer_view: Some(json::Index::new(vertex_data_buffer_view_index)),
            byte_offset: vertex_data_stride,
            count: vertex_count as u32,
            component_type: Valid(json::accessor::GenericComponentType(
                json::accessor::ComponentType::F32,
            )),
            extensions: Default::default(),
            extras: Default::default(),
            type_: Valid(json::accessor::Type::Vec4),
            min: None,
            max: None,
            normalized: false,
            sparse: None,
        });
        attributes_map.insert(
            Valid(json::mesh::Semantic::Colors(0)),
            json::Index::new(accessor_index),
        );
        vertex_data_stride += 4 * 4;
    }

    if zms.uv1_enabled() {
        let accessor_index = root.accessors.len() as u32;
        root.accessors.push(json::Accessor {
            name: Some(format!("{}_UV1", name)),
            buffer_view: Some(json::Index::new(vertex_data_buffer_view_index)),
            byte_offset: vertex_data_stride,
            count: vertex_count as u32,
            component_type: Valid(json::accessor::GenericComponentType(
                json::accessor::ComponentType::F32,
            )),
            extensions: Default::default(),
            extras: Default::default(),
            type_: Valid(json::accessor::Type::Vec2),
            min: None,
            max: None,
            normalized: false,
            sparse: None,
        });
        attributes_map.insert(
            Valid(json::mesh::Semantic::TexCoords(0)),
            json::Index::new(accessor_index),
        );
        vertex_data_stride += 4 * 2;
    }

    if zms.uv2_enabled() {
        let accessor_index = root.accessors.len() as u32;
        root.accessors.push(json::Accessor {
            name: Some(format!("{}_UV2", name)),
            buffer_view: Some(json::Index::new(vertex_data_buffer_view_index)),
            byte_offset: vertex_data_stride,
            count: vertex_count as u32,
            component_type: Valid(json::accessor::GenericComponentType(
                json::accessor::ComponentType::F32,
            )),
            extensions: Default::default(),
            extras: Default::default(),
            type_: Valid(json::accessor::Type::Vec2),
            min: None,
            max: None,
            normalized: false,
            sparse: None,
        });
        attributes_map.insert(
            Valid(json::mesh::Semantic::TexCoords(1)),
            json::Index::new(accessor_index),
        );
        vertex_data_stride += 4 * 2;
    }

    if zms.uv3_enabled() {
        let accessor_index = root.accessors.len() as u32;
        root.accessors.push(json::Accessor {
            name: Some(format!("{}_UV3", name)),
            buffer_view: Some(json::Index::new(vertex_data_buffer_view_index)),
            byte_offset: vertex_data_stride,
            count: vertex_count as u32,
            component_type: Valid(json::accessor::GenericComponentType(
                json::accessor::ComponentType::F32,
            )),
            extensions: Default::default(),
            extras: Default::default(),
            type_: Valid(json::accessor::Type::Vec2),
            min: None,
            max: None,
            normalized: false,
            sparse: None,
        });
        attributes_map.insert(
            Valid(json::mesh::Semantic::TexCoords(2)),
            json::Index::new(accessor_index),
        );
        vertex_data_stride += 4 * 2;
    }

    if zms.uv4_enabled() {
        let accessor_index = root.accessors.len() as u32;
        root.accessors.push(json::Accessor {
            name: Some(format!("{}_UV4", name)),
            buffer_view: Some(json::Index::new(vertex_data_buffer_view_index)),
            byte_offset: vertex_data_stride,
            count: vertex_count as u32,
            component_type: Valid(json::accessor::GenericComponentType(
                json::accessor::ComponentType::F32,
            )),
            extensions: Default::default(),
            extras: Default::default(),
            type_: Valid(json::accessor::Type::Vec2),
            min: None,
            max: None,
            normalized: false,
            sparse: None,
        });
        attributes_map.insert(
            Valid(json::mesh::Semantic::TexCoords(3)),
            json::Index::new(accessor_index),
        );
        vertex_data_stride += 4 * 2;
    }

    if zms.bones_enabled() {
        let accessor_index = root.accessors.len() as u32;
        root.accessors.push(json::Accessor {
            name: Some(format!("{}_BoneWeights", name)),
            buffer_view: Some(json::Index::new(vertex_data_buffer_view_index)),
            byte_offset: vertex_data_stride,
            count: vertex_count as u32,
            component_type: Valid(json::accessor::GenericComponentType(
                json::accessor::ComponentType::F32,
            )),
            extensions: Default::default(),
            extras: Default::default(),
            type_: Valid(json::accessor::Type::Vec4),
            min: None,
            max: None,
            normalized: false,
            sparse: None,
        });
        attributes_map.insert(
            Valid(json::mesh::Semantic::Weights(0)),
            json::Index::new(accessor_index),
        );
        vertex_data_stride += 4 * 4;

        let accessor_index = root.accessors.len() as u32;
        root.accessors.push(json::Accessor {
            name: Some(format!("{}_BoneIndices", name)),
            buffer_view: Some(json::Index::new(vertex_data_buffer_view_index)),
            byte_offset: vertex_data_stride,
            count: vertex_count as u32,
            component_type: Valid(json::accessor::GenericComponentType(
                json::accessor::ComponentType::U16,
            )),
            extensions: Default::default(),
            extras: Default::default(),
            type_: Valid(json::accessor::Type::Vec4),
            min: None,
            max: None,
            normalized: false,
            sparse: None,
        });
        attributes_map.insert(
            Valid(json::mesh::Semantic::Joints(0)),
            json::Index::new(accessor_index),
        );
        vertex_data_stride += 4 * 2;
    }

    let vertex_data_start = binary_data.len() as u32;
    for vertex in zms.vertices.iter() {
        if zms.positions_enabled() {
            binary_data.put_f32_le(vertex.position.x);
            binary_data.put_f32_le(vertex.position.z);
            binary_data.put_f32_le(-vertex.position.y);
        }

        if zms.normals_enabled() {
            binary_data.put_f32_le(vertex.normal.x);
            binary_data.put_f32_le(vertex.normal.z);
            binary_data.put_f32_le(-vertex.normal.y);
        }

        if zms.tangents_enabled() {
            binary_data.put_f32_le(vertex.tangent.x);
            binary_data.put_f32_le(vertex.tangent.z);
            binary_data.put_f32_le(-vertex.tangent.y);
        }

        if zms.colors_enabled() {
            binary_data.put_f32_le(vertex.color.r);
            binary_data.put_f32_le(vertex.color.g);
            binary_data.put_f32_le(vertex.color.b);
            binary_data.put_f32_le(vertex.color.a);
        }

        if zms.uv1_enabled() {
            binary_data.put_f32_le(vertex.uv1.x);
            binary_data.put_f32_le(vertex.uv1.y);
        }

        if zms.uv2_enabled() {
            binary_data.put_f32_le(vertex.uv2.x);
            binary_data.put_f32_le(vertex.uv2.y);
        }

        if zms.uv3_enabled() {
            binary_data.put_f32_le(vertex.uv3.x);
            binary_data.put_f32_le(vertex.uv3.y);
        }

        if zms.uv4_enabled() {
            binary_data.put_f32_le(vertex.uv4.x);
            binary_data.put_f32_le(vertex.uv4.y);
        }

        if zms.bones_enabled() {
            binary_data.put_f32_le(vertex.bone_weights.x);
            binary_data.put_f32_le(vertex.bone_weights.y);
            binary_data.put_f32_le(vertex.bone_weights.z);
            binary_data.put_f32_le(vertex.bone_weights.w);

            binary_data.put_i16_le(zms.bones[vertex.bone_indices.x as usize]);
            binary_data.put_i16_le(zms.bones[vertex.bone_indices.y as usize]);
            binary_data.put_i16_le(zms.bones[vertex.bone_indices.z as usize]);
            binary_data.put_i16_le(zms.bones[vertex.bone_indices.w as usize]);
        }
    }
    let vertex_data_length = binary_data.len() as u32 - vertex_data_start;

    let index_data_start = binary_data.len() as u32;
    let index_data_stride = 2;
    for triangle in zms.indices.iter() {
        binary_data.put_i16_le(triangle.x);
        binary_data.put_i16_le(triangle.y);
        binary_data.put_i16_le(triangle.z);
    }
    let index_data_length = binary_data.len() as u32 - index_data_start;

    root.buffer_views.push(json::buffer::View {
        name: Some(format!("{}_VertexBufferView", name)),
        buffer: json::Index::new(0),
        byte_length: vertex_data_length,
        byte_offset: Some(vertex_data_start),
        byte_stride: Some(vertex_data_stride),
        extensions: Default::default(),
        extras: Default::default(),
        target: Some(Valid(json::buffer::Target::ArrayBuffer)),
    });

    root.buffer_views.push(json::buffer::View {
        name: Some(format!("{}_IndexBufferView", name)),
        buffer: json::Index::new(0),
        byte_length: index_data_length as u32,
        byte_offset: Some(index_data_start),
        byte_stride: Some(index_data_stride),
        extensions: Default::default(),
        extras: Default::default(),
        target: Some(Valid(json::buffer::Target::ArrayBuffer)),
    });

    let indices_accessor_index = root.accessors.len() as u32;
    root.accessors.push(json::Accessor {
        name: Some(format!("{}_Indices", name)),
        buffer_view: Some(json::Index::new(index_data_buffer_view_index)),
        byte_offset: 0,
        count: (3 * zms.indices.len()) as u32,
        component_type: Valid(json::accessor::GenericComponentType(
            json::accessor::ComponentType::U16,
        )),
        extensions: Default::default(),
        extras: Default::default(),
        type_: Valid(json::accessor::Type::Scalar),
        min: None,
        max: None,
        normalized: false,
        sparse: None,
    });

    let primitive = json::mesh::Primitive {
        attributes: attributes_map,
        extensions: Default::default(),
        extras: Default::default(),
        indices: Some(json::Index::new(indices_accessor_index)),
        material: None,
        mode: Valid(json::mesh::Mode::Triangles),
        targets: None,
    };

    let mesh_index = root.meshes.len() as u32;
    root.meshes.push(json::Mesh {
        name: Some(name.into()),
        extensions: Default::default(),
        extras: Default::default(),
        primitives: vec![primitive],
        weights: None,
    });

    mesh_index
}

fn transform_children(zmd: &ZMD, bone_transforms: &mut Vec<Mat4>, bone_index: usize) {
    for (child_id, child_bone) in zmd.bones.iter().enumerate() {
        if child_id == bone_index || child_bone.parent as usize != bone_index {
            continue;
        }

        bone_transforms[child_id] = bone_transforms[bone_index] * bone_transforms[child_id];
        transform_children(zmd, bone_transforms, child_id);
    }
}

fn load_skeleton(
    root: &mut json::Root,
    binary_data: &mut BytesMut,
    name: &str,
    zmd: &ZMD,
) -> json::Index<json::Skin> {
    let bone_node_index_start = root.nodes.len();
    let mut joints = Vec::new();
    let mut bind_pose = Vec::new();

    // Create nodes for each bone
    for i in 0..zmd.bones.len() {
        let bone = &zmd.bones[i];
        root.nodes.push(json::Node {
            name: Some(format!("{}_Bone_{}", name, i)),
            camera: None,
            children: None,
            extensions: Default::default(),
            extras: Default::default(),
            matrix: None,
            mesh: None,
            rotation: Some(UnitQuaternion([
                bone.rotation.x,
                bone.rotation.z,
                -bone.rotation.y,
                bone.rotation.w,
            ])),
            scale: None,
            translation: Some([
                bone.position.x / 100.0,
                bone.position.z / 100.0,
                -bone.position.y / 100.0,
            ]),
            skin: None,
            weights: None,
        });
        joints.push(json::Index::new(bone_node_index_start as u32 + i as u32));

        let translation = Vec3::new(bone.position.x, bone.position.z, -bone.position.y) / 100.0;
        let rotation = Quat::from_xyzw(
            bone.rotation.x,
            bone.rotation.z,
            -bone.rotation.y,
            bone.rotation.w,
        );
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
            children.push(json::Index::new(bone_node_index_start as u32 + i as u32));
        } else {
            parent.children = Some(vec![json::Index::new(
                bone_node_index_start as u32 + i as u32,
            )]);
        }
    }

    // Calculate inverse bind pose
    transform_children(zmd, &mut bind_pose, 0);
    let inverse_bind_pose: Vec<Mat4> = bind_pose.iter().map(|x| x.inverse()).collect();

    let skeleton_data_start = binary_data.len() as u32;
    for mtx in inverse_bind_pose.iter() {
        for f in mtx.to_cols_array() {
            binary_data.put_f32_le(f);
        }
    }
    let skeleton_data_length = binary_data.len() as u32 - skeleton_data_start;
    let skeleton_data_stride = 4 * 16;

    let buffer_view_index = root.buffer_views.len() as u32;
    root.buffer_views.push(json::buffer::View {
        name: Some(format!("{}_SkeletonBufferView", name)),
        buffer: json::Index::new(0),
        byte_length: skeleton_data_length as u32,
        byte_offset: Some(skeleton_data_start),
        byte_stride: Some(skeleton_data_stride),
        extensions: Default::default(),
        extras: Default::default(),
        target: Some(Valid(json::buffer::Target::ArrayBuffer)),
    });

    let accessor_index = root.accessors.len() as u32;
    root.accessors.push(json::Accessor {
        name: Some(format!("{}_SkeletonAccessor", name)),
        buffer_view: Some(json::Index::new(buffer_view_index)),
        byte_offset: 0,
        count: inverse_bind_pose.len() as u32,
        component_type: Valid(json::accessor::GenericComponentType(
            json::accessor::ComponentType::F32,
        )),
        extensions: Default::default(),
        extras: Default::default(),
        type_: Valid(json::accessor::Type::Mat4),
        min: None,
        max: None,
        normalized: false,
        sparse: None,
    });

    let skin_index = root.skins.len() as u32;
    root.skins.push(json::Skin {
        name: Some(name.to_string()),
        extensions: Default::default(),
        extras: Default::default(),
        inverse_bind_matrices: Some(json::Index::new(accessor_index)),
        skeleton: Some(joints[0]),
        joints,
    });
    json::Index::new(skin_index)
}

fn load_skeletal_animation(
    root: &mut json::Root,
    binary_data: &mut BytesMut,
    name: &str,
    skin_index: json::Index<json::Skin>,
    zmo: &ZMO,
) {
    let mut channels = Vec::new();
    let mut samplers = Vec::new();

    let keyframe_time_start = binary_data.len() as u32;
    let fps = zmo.fps as f32;
    for i in 0..zmo.frames {
        binary_data.put_f32_le(i as f32 / fps)
    }
    let keyframe_time_length = binary_data.len() as u32 - keyframe_time_start;
    let keyframe_time_stride = 4;

    let buffer_view_index = root.buffer_views.len() as u32;
    root.buffer_views.push(json::buffer::View {
        name: Some(format!("{}_KeyframeTimesBuferView", name)),
        buffer: json::Index::new(0),
        byte_length: keyframe_time_length as u32,
        byte_offset: Some(keyframe_time_start),
        byte_stride: Some(keyframe_time_stride),
        extensions: Default::default(),
        extras: Default::default(),
        target: Some(Valid(json::buffer::Target::ArrayBuffer)),
    });

    let keyframe_time_accessor_index = json::Index::new(root.accessors.len() as u32);
    root.accessors.push(json::Accessor {
        name: Some(format!("{}_KeyframeTimesAccessor", name)),
        buffer_view: Some(json::Index::new(buffer_view_index)),
        byte_offset: 0,
        count: zmo.frames,
        component_type: Valid(json::accessor::GenericComponentType(
            json::accessor::ComponentType::F32,
        )),
        extensions: Default::default(),
        extras: Default::default(),
        type_: Valid(json::accessor::Type::Scalar),
        min: None,
        max: None,
        normalized: false,
        sparse: None,
    });

    for (channel_id, channel) in zmo.channels.iter().enumerate() {
        if !matches!(
            channel.typ,
            ChannelType::Position | ChannelType::Rotation | ChannelType::Scale
        ) {
            continue;
        }

        let keyframe_data_start = binary_data.len() as u32;
        let keyframe_data_stride = match &channel.frames {
            ChannelData::Position(positions) => {
                for position in positions.iter() {
                    binary_data.put_f32_le(position.x / 100.0);
                    binary_data.put_f32_le(position.z / 100.0);
                    binary_data.put_f32_le(-position.y / 100.0);
                }

                4 * 3
            }
            ChannelData::Rotation(rotations) => {
                for rotation in rotations.iter() {
                    binary_data.put_f32_le(rotation.x);
                    binary_data.put_f32_le(rotation.z);
                    binary_data.put_f32_le(-rotation.y);
                    binary_data.put_f32_le(rotation.w);
                }

                4 * 4
            }
            ChannelData::Scale(scales) => {
                for scale in scales.iter() {
                    binary_data.put_f32_le(*scale);
                    binary_data.put_f32_le(*scale);
                    binary_data.put_f32_le(*scale);
                }

                4 * 3
            }
            _ => unreachable!(),
        };
        let keyframe_data_length = binary_data.len() as u32 - keyframe_data_start;

        let buffer_view_index = root.buffer_views.len() as u32;
        root.buffer_views.push(json::buffer::View {
            name: Some(format!("{}_Channel{}_DataBufferView", name, channel_id)),
            buffer: json::Index::new(0),
            byte_length: keyframe_data_length as u32,
            byte_offset: Some(keyframe_data_start),
            byte_stride: Some(keyframe_data_stride),
            extensions: Default::default(),
            extras: Default::default(),
            target: Some(Valid(json::buffer::Target::ArrayBuffer)),
        });

        let keyframe_data_accessor_index = json::Index::new(root.accessors.len() as u32);
        root.accessors.push(json::Accessor {
            name: Some(format!("{}_Channel{}_DataAccessor", name, channel_id)),
            buffer_view: Some(json::Index::new(buffer_view_index)),
            byte_offset: 0,
            count: zmo.frames,
            component_type: Valid(json::accessor::GenericComponentType(
                json::accessor::ComponentType::F32,
            )),
            extensions: Default::default(),
            extras: Default::default(),
            type_: Valid(if matches!(channel.typ, ChannelType::Rotation) {
                json::accessor::Type::Vec4
            } else {
                json::accessor::Type::Vec3
            }),
            min: None,
            max: None,
            normalized: false,
            sparse: None,
        });

        let sampler_index = json::Index::new(samplers.len() as u32);
        samplers.push(json::animation::Sampler {
            input: keyframe_time_accessor_index,
            interpolation: Valid(Interpolation::Linear),
            output: keyframe_data_accessor_index,
            extensions: Default::default(),
            extras: Default::default(),
        });

        channels.push(json::animation::Channel {
            sampler: sampler_index,
            target: json::animation::Target {
                node: root.get(skin_index).unwrap().joints[channel.index as usize],
                path: Valid(match channel.typ {
                    ChannelType::Position => json::animation::Property::Translation,
                    ChannelType::Rotation => json::animation::Property::Rotation,
                    ChannelType::Scale => json::animation::Property::Scale,
                    _ => unreachable!(),
                }),
                extensions: Default::default(),
                extras: Default::default(),
            },
            extensions: Default::default(),
            extras: Default::default(),
        });
    }

    root.animations.push(json::Animation {
        extensions: Default::default(),
        extras: Default::default(),
        channels,
        name: Some(name.to_string()),
        samplers,
    });
}

fn main() {
    let matches = clap::Command::new("rose-gltf")
        .arg(
            clap::Arg::new("out")
                .short('o')
                .long("out")
                .takes_value(true),
        )
        .arg(
            clap::Arg::new("input-files")
                .takes_value(true)
                .multiple_values(true),
        )
        .get_matches();

    let output_file_path = PathBuf::from(matches.value_of("out").unwrap_or("out.glb"));
    let input_files = matches
        .values_of("input-files")
        .expect("No input files specified");

    let mut root = json::Root::default();
    let mut binary_data = BytesMut::with_capacity(8 * 1024 * 1024);
    let mut scene = json::Scene {
        name: None,
        extensions: Default::default(),
        extras: Default::default(),
        nodes: Default::default(),
    };
    let mut skin_index = None;

    for input_file in input_files {
        let file_path = PathBuf::from(input_file);
        let file_name = file_path.file_name().unwrap().to_str().unwrap().to_string();
        let file_extension = file_path
            .extension()
            .unwrap()
            .to_ascii_lowercase()
            .to_str()
            .unwrap()
            .to_string();

        match file_extension.as_str() {
            "zmd" => {
                let zmd = ZMD::from_path(&file_path).expect("Failed to load ZMD");

                skin_index = Some(load_skeleton(&mut root, &mut binary_data, &file_name, &zmd));
            }
            "zmo" => {
                let zmo = ZMO::from_path(&file_path).expect("Failed to load ZMO");

                if let Some(skin_index) = skin_index {
                    load_skeletal_animation(
                        &mut root,
                        &mut binary_data,
                        &file_name,
                        skin_index,
                        &zmo,
                    );
                }
            }
            "zms" => {
                let zms = ZMS::from_path(&file_path).expect("Failed to load ZMS");

                let mesh_index = load_mesh(&mut root, &mut binary_data, &file_name, &zms);
                let node_index = root.nodes.len() as u32;
                root.nodes.push(json::Node {
                    name: Some(format!("{}_Node", file_name)),
                    camera: None,
                    children: None,
                    extensions: Default::default(),
                    extras: Default::default(),
                    matrix: None,
                    mesh: Some(json::Index::new(mesh_index)),
                    rotation: None,
                    scale: None,
                    translation: None,
                    skin: if zms.bones_enabled() {
                        skin_index
                    } else {
                        None
                    },
                    weights: None,
                });
                scene.nodes.push(json::Index::new(node_index));
            }
            unknown => {
                panic!("Unsupported file extension {}", unknown);
            }
        }
    }

    root.buffers.push(json::Buffer {
        name: None,
        byte_length: binary_data.len() as u32,
        extensions: Default::default(),
        extras: Default::default(),
        uri: None,
    });

    // Data must be padded to 4
    let binary_length = binary_data.len() as u32;
    while binary_data.len() % 4 != 0 {
        binary_data.put_u8(0);
    }

    let json_string = json::serialize::to_string(&root).expect("Serialization error");
    let json_length = (json_string.len() as u32 + 3) & !3;
    let glb = gltf::binary::Glb {
        header: gltf::binary::Header {
            magic: *b"glTF",
            version: 2,
            length: json_length + binary_length,
        },
        bin: Some(Cow::Borrowed(binary_data.as_ref())),
        json: Cow::Owned(json_string.into_bytes()),
    };
    let writer = std::fs::File::create(output_file_path).expect("I/O error");
    glb.to_writer(writer).expect("glTF binary output error");
}
