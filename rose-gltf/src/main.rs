use bytes::{BufMut, BytesMut};
use glam::{Mat4, Quat, Vec3};
use gltf::{animation::Interpolation, Semantic};
use gltf_json as json;
use image::{codecs::png::PngEncoder, ImageEncoder, Pixel};
use json::{
    accessor,
    material::{
        AlphaCutoff, AlphaMode, EmissiveFactor, PbrBaseColorFactor, PbrMetallicRoughness,
        StrengthFactor,
    },
    scene::UnitQuaternion,
    validation::Checked::{self, Valid},
    Index, Material,
};
use roselib::{
    files::{
        him::Heightmap,
        ifo::MapData,
        til::Tilemap,
        zmo::{ChannelData, ChannelType},
        HIM, IFO, STB, TIL, ZMD, ZMO, ZMS, ZON, ZSC, zon::ZoneTileRotation,
    },
    io::RoseFile,
    utils::{Quaternion, Vector3},
};
use serde_json::json;
use std::{
    borrow::Cow,
    collections::HashMap,
    ffi::OsStr,
    io::Cursor,
    path::{Path, PathBuf},
};

#[derive(Clone)]
struct MeshData {
    pub attributes: HashMap<Checked<Semantic>, Index<accessor::Accessor>>,
    pub indices: Index<accessor::Accessor>,
}

fn load_mesh_data(
    root: &mut json::Root,
    binary_data: &mut BytesMut,
    name: &str,
    zms: &ZMS,
) -> MeshData {
    let mut attributes_map = HashMap::new();
    let mut vertex_data_stride = 0;
    let vertex_count = zms.vertices.len();
    let vertex_data_buffer_view_index = root.buffer_views.len() as u32;
    let index_data_buffer_view_index = vertex_data_buffer_view_index + 1;

    if zms.positions_enabled() {
        let mut min_pos = Vec3::new(
            zms.vertices[0].position.x,
            zms.vertices[0].position.z,
            -zms.vertices[0].position.y,
        );
        let mut max_pos = Vec3::new(
            zms.vertices[0].position.x,
            zms.vertices[0].position.z,
            -zms.vertices[0].position.y,
        );
        for vertex in zms.vertices.iter() {
            min_pos.x = min_pos.x.min(vertex.position.x);
            min_pos.y = min_pos.y.min(vertex.position.z);
            min_pos.z = min_pos.z.min(-vertex.position.y);

            max_pos.x = max_pos.x.max(vertex.position.x);
            max_pos.y = max_pos.y.max(vertex.position.z);
            max_pos.z = max_pos.z.max(-vertex.position.y);
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
            min: Some(json!([min_pos.x, min_pos.y, min_pos.z])),
            max: Some(json!([max_pos.x, max_pos.y, max_pos.z])),
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

            binary_data.put_i16_le(if vertex.bone_weights.x == 0.0 {
                0
            } else {
                zms.bones[vertex.bone_indices.x as usize]
            });
            binary_data.put_i16_le(if vertex.bone_weights.y == 0.0 {
                0
            } else {
                zms.bones[vertex.bone_indices.y as usize]
            });
            binary_data.put_i16_le(if vertex.bone_weights.z == 0.0 {
                0
            } else {
                zms.bones[vertex.bone_indices.z as usize]
            });
            binary_data.put_i16_le(if vertex.bone_weights.w == 0.0 {
                0
            } else {
                zms.bones[vertex.bone_indices.w as usize]
            });
        }
    }
    let vertex_data_length = binary_data.len() as u32 - vertex_data_start;

    let index_data_start = binary_data.len() as u32;
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
        byte_length: index_data_length,
        byte_offset: Some(index_data_start),
        byte_stride: None,
        extensions: Default::default(),
        extras: Default::default(),
        target: Some(Valid(json::buffer::Target::ElementArrayBuffer)),
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

    MeshData {
        attributes: attributes_map,
        indices: json::Index::new(indices_accessor_index),
    }
}

fn load_mesh(root: &mut json::Root, binary_data: &mut BytesMut, name: &str, zms: &ZMS) -> u32 {
    let mesh_data = load_mesh_data(root, binary_data, name, zms);
    let mesh_index = root.meshes.len() as u32;
    root.meshes.push(json::Mesh {
        name: Some(name.into()),
        extensions: Default::default(),
        extras: Default::default(),
        primitives: vec![json::mesh::Primitive {
            attributes: mesh_data.attributes,
            extensions: Default::default(),
            extras: Default::default(),
            indices: Some(mesh_data.indices),
            material: None,
            mode: Valid(json::mesh::Mode::Triangles),
            targets: None,
        }],
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

    // Add root node to scene
    root.scenes[0]
        .nodes
        .push(json::Index::new(root.nodes.len() as u32));

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

        root.nodes.push(json::Node {
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

        joints.push(json::Index::new(bone_node_index_start as u32 + i as u32));
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

    let buffer_view_index = root.buffer_views.len() as u32;
    root.buffer_views.push(json::buffer::View {
        name: Some(format!("{}_SkeletonBufferView", name)),
        buffer: json::Index::new(0),
        byte_length: skeleton_data_length,
        byte_offset: Some(skeleton_data_start),
        byte_stride: None,
        extensions: Default::default(),
        extras: Default::default(),
        target: None,
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

    let buffer_view_index = root.buffer_views.len() as u32;
    root.buffer_views.push(json::buffer::View {
        name: Some(format!("{}_KeyframeTimesBuferView", name)),
        buffer: json::Index::new(0),
        byte_length: keyframe_time_length,
        byte_offset: Some(keyframe_time_start),
        byte_stride: None,
        extensions: Default::default(),
        extras: Default::default(),
        target: None,
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
        min: Some(json!([0.0])),
        max: Some(json!([(zmo.frames - 1) as f32 / fps])),
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
        match &channel.frames {
            ChannelData::Position(positions) => {
                for position in positions.iter() {
                    binary_data.put_f32_le(position.x / 100.0);
                    binary_data.put_f32_le(position.z / 100.0);
                    binary_data.put_f32_le(-position.y / 100.0);
                }
            }
            ChannelData::Rotation(rotations) => {
                for rotation in rotations.iter() {
                    binary_data.put_f32_le(rotation.x);
                    binary_data.put_f32_le(rotation.z);
                    binary_data.put_f32_le(-rotation.y);
                    binary_data.put_f32_le(rotation.w);
                }
            }
            ChannelData::Scale(scales) => {
                for scale in scales.iter() {
                    binary_data.put_f32_le(*scale);
                    binary_data.put_f32_le(*scale);
                    binary_data.put_f32_le(*scale);
                }
            }
            _ => unreachable!(),
        };
        let keyframe_data_length = binary_data.len() as u32 - keyframe_data_start;

        let buffer_view_index = root.buffer_views.len() as u32;
        root.buffer_views.push(json::buffer::View {
            name: Some(format!("{}_Channel{}_DataBufferView", name, channel_id)),
            buffer: json::Index::new(0),
            byte_length: keyframe_data_length,
            byte_offset: Some(keyframe_data_start),
            byte_stride: None,
            extensions: Default::default(),
            extras: Default::default(),
            target: None,
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

fn find_assets_root_path(file_path: &Path) -> Option<PathBuf> {
    let mut path = file_path;
    while let Some(parent_path) = path.parent() {
        if parent_path
            .file_name()
            .map_or(false, |s| OsStr::new("3ddata").eq_ignore_ascii_case(s))
        {
            return parent_path.parent().map(|p| p.to_path_buf());
        }

        path = parent_path;
    }

    None
}

fn convert_position(position: Vector3<f32>) -> [f32; 3] {
    [position.x / 100.0, position.z / 100.0, -position.y / 100.0]
}

fn convert_rotation(rotation: Quaternion) -> UnitQuaternion {
    UnitQuaternion([rotation.x, rotation.z, -rotation.y, rotation.w])
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

    let mut binary_data = BytesMut::with_capacity(8 * 1024 * 1024);
    let mut root = json::Root::default();
    root.scenes.push(json::Scene {
        name: None,
        extensions: Default::default(),
        extras: Default::default(),
        nodes: Default::default(),
    });

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
                root.scenes[0].nodes.push(json::Index::new(node_index));
            }
            "zon" => {
                let map_path = file_path
                    .parent()
                    .expect("Could not find map path")
                    .to_path_buf();
                let assets_path =
                    find_assets_root_path(&file_path).expect("Could not find root assets path");
                let relative_zon_path = file_path.strip_prefix(&assets_path).unwrap();
                let list_zone = STB::from_path(&assets_path.join("3ddata/stb/list_zone.stb"))
                    .expect("Failed to load list_zone.stb");
                let zone_id = (|| {
                    for row in 1..list_zone.rows() {
                        if let Some(row_zon) = list_zone.value(row, 2) {
                            if Path::new(&row_zon.to_ascii_lowercase()) == relative_zon_path {
                                return Some(row);
                            }
                        }
                    }
                    None
                })()
                .expect("Could not find zone id");

                struct ObjectList {
                    pub zsc: ZSC,
                    pub materials: HashMap<u16, Index<json::Material>>,
                    pub meshes: HashMap<u16, MeshData>,
                }
                impl ObjectList {
                    pub fn new(zsc: ZSC) -> Self {
                        Self {
                            materials: HashMap::with_capacity(zsc.materials.len()),
                            meshes: HashMap::with_capacity(zsc.meshes.len()),
                            zsc,
                        }
                    }

                    pub fn load_object(
                        &mut self,
                        name_prefix: &str,
                        object_id: usize,
                        root: &mut json::Root,
                        binary_data: &mut BytesMut,
                        assets_path: &Path,
                    ) {
                        let object = self.zsc.objects.get(object_id).expect("Invalid object id");
                        for part in object.parts.iter() {
                            if let Some(material_data) = self.load_material(
                                name_prefix,
                                part.material_id,
                                root,
                                binary_data,
                                assets_path,
                            ) {
                                self.materials.insert(part.material_id, material_data);
                            }

                            if let Some(mesh_data) = self.load_mesh(
                                name_prefix,
                                part.mesh_id,
                                root,
                                binary_data,
                                assets_path,
                            ) {
                                self.meshes.insert(part.mesh_id, mesh_data);
                            }
                        }
                    }

                    pub fn load_mesh(
                        &self,
                        name_prefix: &str,
                        mesh_id: u16,
                        root: &mut json::Root,
                        binary_data: &mut BytesMut,
                        assets_path: &Path,
                    ) -> Option<MeshData> {
                        if self.meshes.contains_key(&mesh_id) {
                            // Already loaded
                            return None;
                        }

                        let zms =
                            ZMS::from_path(&assets_path.join(&self.zsc.meshes[mesh_id as usize]))
                                .expect("Failed to load ZMS");
                        Some(load_mesh_data(
                            root,
                            binary_data,
                            &format!("{}_mesh_{}", name_prefix, mesh_id),
                            &zms,
                        ))
                    }

                    pub fn load_material(
                        &self,
                        name_prefix: &str,
                        material_id: u16,
                        root: &mut json::Root,
                        binary_data: &mut BytesMut,
                        assets_path: &Path,
                    ) -> Option<Index<json::Material>> {
                        if self.materials.contains_key(&material_id) {
                            // Already loaded
                            return None;
                        }

                        let material = self.zsc.materials.get(material_id as usize).unwrap();

                        let img = image::open(assets_path.join(&material.path))
                            .expect("Failed to load DDS");
                        let mut png_buffer: Vec<u8> = Vec::new();
                        img.write_to(
                            &mut Cursor::new(&mut png_buffer),
                            image::ImageOutputFormat::Png,
                        )
                        .expect("Failed to write PNG");

                        let texture_data_start = binary_data.len();
                        binary_data.put_slice(&png_buffer);

                        let buffer_index = Index::new(root.buffer_views.len() as u32);
                        root.buffer_views.push(json::buffer::View {
                            name: Some(format!(
                                "{}_material_{}_image_buffer",
                                name_prefix, material_id
                            )),
                            buffer: json::Index::new(0),
                            byte_length: png_buffer.len() as u32,
                            byte_offset: Some(texture_data_start as u32),
                            byte_stride: None,
                            extensions: Default::default(),
                            extras: Default::default(),
                            target: Some(Valid(json::buffer::Target::ArrayBuffer)),
                        });

                        let image_index = Index::new(root.images.len() as u32);
                        root.images.push(json::Image {
                            name: Some(format!("{}_material_{}_image", name_prefix, material_id)),
                            buffer_view: Some(buffer_index),
                            mime_type: Some(json::image::MimeType("image/png".into())),
                            uri: None,
                            extensions: None,
                            extras: Default::default(),
                        });

                        let texture_index = Index::new(root.textures.len() as u32);
                        root.textures.push(json::Texture {
                            name: Some(format!("{}_material_{}_texture", name_prefix, material_id)),
                            sampler: Some(Index::new(0)),
                            source: image_index,
                            extensions: None,
                            extras: Default::default(),
                        });

                        let material_index = Index::new(root.materials.len() as u32);
                        root.materials.push(json::Material {
                            name: Some(format!("{}_material_{}", name_prefix, material_id)),
                            alpha_cutoff: if material.alpha_test_enabled {
                                Some(AlphaCutoff(material.alpha_ref as f32 / 100.0))
                            } else {
                                None
                            },
                            alpha_mode: json::validation::Checked::Valid(
                                if material.alpha_test_enabled {
                                    AlphaMode::Mask
                                } else if material.alpha_enabled {
                                    AlphaMode::Blend
                                } else {
                                    AlphaMode::Opaque
                                },
                            ),
                            double_sided: material.two_sided,
                            pbr_metallic_roughness: PbrMetallicRoughness {
                                base_color_factor: PbrBaseColorFactor([1.0, 1.0, 1.0, 1.0]),
                                base_color_texture: Some(json::texture::Info {
                                    index: texture_index,
                                    tex_coord: 0,
                                    extensions: None,
                                    extras: Default::default(),
                                }),
                                metallic_factor: StrengthFactor(0.0),
                                roughness_factor: StrengthFactor(0.0),
                                metallic_roughness_texture: None,
                                extensions: None,
                                extras: Default::default(),
                            },
                            normal_texture: None,
                            occlusion_texture: None,
                            emissive_texture: None,
                            emissive_factor: EmissiveFactor([0.0, 0.0, 0.0]),
                            extensions: None,
                            extras: Default::default(),
                        });
                        Some(material_index)
                    }
                }

                let zon = ZON::from_path(&file_path).expect("Failed to load ZON");
                let mut deco = ObjectList::new(
                    ZSC::from_path(
                        &assets_path.join(Path::new(list_zone.value(zone_id, 12).unwrap())),
                    )
                    .expect("Failed to read deco zsc"),
                );
                let mut cnst = ObjectList::new(
                    ZSC::from_path(
                        &assets_path.join(Path::new(list_zone.value(zone_id, 13).unwrap())),
                    )
                    .expect("Failed to read cnst zsc"),
                );

                struct BlockData {
                    pub block_x: i32,
                    pub block_y: i32,
                    pub ifo: MapData,
                    pub him: Heightmap,
                    pub til: Tilemap,
                }
                let mut blocks = Vec::new();

                for block_y in 0..64 {
                    for block_x in 0..64 {
                        let ifo =
                            IFO::from_path(&map_path.join(format!("{}_{}.ifo", block_x, block_y)));
                        let him =
                            HIM::from_path(&map_path.join(format!("{}_{}.him", block_x, block_y)));
                        let til =
                            TIL::from_path(&map_path.join(format!("{}_{}.til", block_x, block_y)));
                        if let (Ok(ifo), Ok(him), Ok(til)) = (ifo, him, til) {
                            blocks.push(BlockData {
                                block_x,
                                block_y,
                                ifo,
                                him,
                                til,
                            });
                        }
                    }
                }

                // Load a sampler for every texture to use
                root.samplers.push(json::texture::Sampler {
                    name: Some("default_sampler".to_string()),
                    mag_filter: Some(json::validation::Checked::Valid(
                        json::texture::MagFilter::Linear,
                    )),
                    min_filter: Some(json::validation::Checked::Valid(
                        json::texture::MinFilter::LinearMipmapLinear,
                    )),
                    wrap_s: json::validation::Checked::Valid(
                        json::texture::WrappingMode::ClampToEdge,
                    ),
                    wrap_t: json::validation::Checked::Valid(
                        json::texture::WrappingMode::ClampToEdge,
                    ),
                    extensions: None,
                    extras: Default::default(),
                });

                // Load all meshes and materials from used objects
                for block in blocks.iter() {
                    for block_objects in block.ifo.objects.iter() {
                        deco.load_object(
                            "deco",
                            block_objects.object_id as usize,
                            &mut root,
                            &mut binary_data,
                            &assets_path,
                        );
                    }

                    for block_objects in block.ifo.buildings.iter() {
                        cnst.load_object(
                            "cnst",
                            block_objects.object_id as usize,
                            &mut root,
                            &mut binary_data,
                            &assets_path,
                        );
                    }
                }

                let splatmap_size = 1024;
                let splatmap_tile_size = splatmap_size / 16;
                let mut tile_textures = Vec::with_capacity(zon.textures.len());

                for tile_texure_path in zon.textures.iter() {
                    if tile_texure_path == "end" {
                        break;
                    }
                    let mut img = image::open(assets_path.join(tile_texure_path))
                        .expect("Failed to load DDS");

                    if img.width() != splatmap_tile_size {
                        img = img.resize(
                            splatmap_tile_size,
                            splatmap_tile_size,
                            image::imageops::FilterType::Triangle,
                        );
                    }

                    tile_textures.push(img.to_rgba8());
                }

                // Spawn all block nodes
                for block in blocks.iter() {
                    let mut splatmap = image::RgbImage::new(splatmap_size, splatmap_size);

                    {
                        // Create a heightmap
                        let mut positions = Vec::new();
                        let mut normals = Vec::new();
                        let mut uvs = Vec::new();
                        let mut indices = Vec::new();

                        for tile_x in 0..16 {
                            for tile_y in 0..16 {
                                let tile =
                                    &zon.tiles[block.til.tiles[tile_y][tile_x].tile_id as usize];
                                let tile_index1 = (tile.layer1 + tile.offset1) as usize;
                                let tile_index2 = (tile.layer2 + tile.offset2) as usize;
                                let tile_image1 = tile_textures.get(tile_index1).unwrap();
                                let tile_image2 = tile_textures.get(tile_index2).unwrap();

                                fn lerp(a: u8, b: u8, x: u8) -> u8 {
                                    ((a as u16 * (256 - x as u16) + b as u16 * x as u16) >> 8) as u8
                                }

                                let dst_x = tile_x as u32 * splatmap_tile_size;
                                let dst_y = tile_y as u32 * splatmap_tile_size;
                                match tile.rotation {
                                    ZoneTileRotation::Unknown | ZoneTileRotation::None =>
                                    {
                                        for src_y in 0..splatmap_tile_size {
                                            for src_x in 0..splatmap_tile_size {
                                                let pixel1 = tile_image1.get_pixel(src_x, src_y);
                                                let pixel2 = tile_image2.get_pixel(src_x, src_y);
                                                splatmap.put_pixel(
                                                    dst_x + src_x,
                                                    dst_y + src_y,
                                                    image::Rgb([
                                                        lerp(pixel1[0], pixel2[0], pixel2[3]),
                                                        lerp(pixel1[1], pixel2[1], pixel2[3]),
                                                        lerp(pixel1[2], pixel2[2], pixel2[3]),
                                                    ]),
                                                );
                                            }
                                        }
                                    },
                                    ZoneTileRotation::FlipHorizontal =>
                                    {
                                        for src_y in 0..splatmap_tile_size {
                                            for src_x in 0..splatmap_tile_size {
                                                let pixel1 = tile_image1.get_pixel(src_x, src_y);
                                                let pixel2 = tile_image2.get_pixel(splatmap_tile_size - 1 - src_x, src_y);
                                                splatmap.put_pixel(
                                                    dst_x + src_x,
                                                    dst_y + src_y,
                                                    image::Rgb([
                                                        lerp(pixel1[0], pixel2[0], pixel2[3]),
                                                        lerp(pixel1[1], pixel2[1], pixel2[3]),
                                                        lerp(pixel1[2], pixel2[2], pixel2[3]),
                                                    ]),
                                                );
                                            }
                                        }
                                    },
                                    ZoneTileRotation::FlipVertical =>
                                    {
                                        for src_y in 0..splatmap_tile_size {
                                            for src_x in 0..splatmap_tile_size {
                                                let pixel1 = tile_image1.get_pixel(src_x, src_y);
                                                let pixel2 = tile_image2.get_pixel(src_x, splatmap_tile_size - 1 - src_y);
                                                splatmap.put_pixel(
                                                    dst_x + src_x,
                                                    dst_y + src_y,
                                                    image::Rgb([
                                                        lerp(pixel1[0], pixel2[0], pixel2[3]),
                                                        lerp(pixel1[1], pixel2[1], pixel2[3]),
                                                        lerp(pixel1[2], pixel2[2], pixel2[3]),
                                                    ]),
                                                );
                                            }
                                        }
                                    },
                                    ZoneTileRotation::Flip =>
                                    {
                                        for src_y in 0..splatmap_tile_size {
                                            for src_x in 0..splatmap_tile_size {
                                                let pixel1 = tile_image1.get_pixel(src_x, src_y);
                                                let pixel2 = tile_image2.get_pixel(splatmap_tile_size - 1 - src_x,splatmap_tile_size - 1 - src_y);
                                                splatmap.put_pixel(
                                                    dst_x + src_x,
                                                    dst_y + src_y,
                                                    image::Rgb([
                                                        lerp(pixel1[0], pixel2[0], pixel2[3]),
                                                        lerp(pixel1[1], pixel2[1], pixel2[3]),
                                                        lerp(pixel1[2], pixel2[2], pixel2[3]),
                                                    ]),
                                                );
                                            }
                                        }
                                    },
                                    ZoneTileRotation::Clockwise90 =>
                                    {
                                        for src_y in 0..splatmap_tile_size {
                                            for src_x in 0..splatmap_tile_size {
                                                let pixel1 = tile_image1.get_pixel(src_x, src_y);
                                                let pixel2 = tile_image2.get_pixel(src_y, splatmap_tile_size - 1 - src_x);
                                                splatmap.put_pixel(
                                                    dst_x + src_x,
                                                    dst_y + src_y,
                                                    image::Rgb([
                                                        lerp(pixel1[0], pixel2[0], pixel2[3]),
                                                        lerp(pixel1[1], pixel2[1], pixel2[3]),
                                                        lerp(pixel1[2], pixel2[2], pixel2[3]),
                                                    ]),
                                                );
                                            }
                                        }
                                    },
                                    ZoneTileRotation::CounterClockwise90 =>
                                    {
                                        for src_y in 0..splatmap_tile_size {
                                            for src_x in 0..splatmap_tile_size {
                                                let pixel1 = tile_image1.get_pixel(src_x, src_y);
                                                let pixel2 = tile_image2.get_pixel(src_y, src_x);
                                                splatmap.put_pixel(
                                                    dst_x + src_x,
                                                    dst_y + src_y,
                                                    image::Rgb([
                                                        lerp(pixel1[0], pixel2[0], pixel2[3]),
                                                        lerp(pixel1[1], pixel2[1], pixel2[3]),
                                                        lerp(pixel1[2], pixel2[2], pixel2[3]),
                                                    ]),
                                                );
                                            }
                                        }
                                    },
                                }

                                let tile_indices_base = positions.len() as u16;
                                let tile_offset_x = tile_x as f32 * 4.0 * 2.5;
                                let tile_offset_y = tile_y as f32 * 4.0 * 2.5;

                                fn get_height(him: &Heightmap, x: i32, y: i32) -> f32 {
                                    let x = i32::clamp(x, 0, him.width - 1) as usize;
                                    let y = i32::clamp(y, 0, him.length - 1) as usize;
                                    him.heights[y * him.width as usize + x] / 100.0
                                }

                                for y in 0..5 {
                                    for x in 0..5 {
                                        let heightmap_x = x + tile_x as i32 * 4;
                                        let heightmap_y = y + tile_y as i32 * 4;
                                        let height =
                                            get_height(&block.him, heightmap_x, heightmap_y);
                                        let height_l =
                                            get_height(&block.him, heightmap_x - 1, heightmap_y);
                                        let height_r =
                                            get_height(&block.him, heightmap_x + 1, heightmap_y);
                                        let height_t =
                                            get_height(&block.him, heightmap_x, heightmap_y - 1);
                                        let height_b =
                                            get_height(&block.him, heightmap_x, heightmap_y + 1);
                                        let normal = Vec3::new(
                                            (height_l - height_r) / 2.0,
                                            1.0,
                                            (height_t - height_b) / 2.0,
                                        )
                                        .normalize();

                                        positions.push([
                                            tile_offset_x + x as f32 * 2.5,
                                            height,
                                            tile_offset_y + y as f32 * 2.5,
                                        ]);
                                        normals.push([normal.x, normal.y, normal.z]);
                                        uvs.push([
                                            (tile_x as f32 * 4.0 + x as f32) / 64.0,
                                            (tile_y as f32 * 4.0 + y as f32) / 64.0,
                                        ]);

                                        // tile_ids.push(tile_array_index1 | tile_array_index2 << 8 | tile_rotation << 16);
                                    }
                                }

                                for y in 0..(5 - 1) {
                                    for x in 0..(5 - 1) {
                                        let start = tile_indices_base + y * 5 + x;
                                        indices.push(start);
                                        indices.push(start + 5);
                                        indices.push(start + 1);

                                        indices.push(start + 1);
                                        indices.push(start + 5);
                                        indices.push(start + 1 + 5);
                                    }
                                }
                            }
                        }

                        let mut png_buffer: Vec<u8> = Vec::new();
                        splatmap
                            .write_to(
                                &mut Cursor::new(&mut png_buffer),
                                image::ImageOutputFormat::Png,
                            )
                            .expect("Failed to write PNG");
                        let heightmap_material = {
                            let texture_data_start = binary_data.len();
                            binary_data.put_slice(&png_buffer);

                            let buffer_index = Index::new(root.buffer_views.len() as u32);
                            root.buffer_views.push(json::buffer::View {
                                name: Some(format!(
                                    "{}_{}_tilemap_image_buffer",
                                    block.block_x, block.block_y,
                                )),
                                buffer: json::Index::new(0),
                                byte_length: png_buffer.len() as u32,
                                byte_offset: Some(texture_data_start as u32),
                                byte_stride: None,
                                extensions: Default::default(),
                                extras: Default::default(),
                                target: Some(Valid(json::buffer::Target::ArrayBuffer)),
                            });

                            let image_index = Index::new(root.images.len() as u32);
                            root.images.push(json::Image {
                                name: Some(format!(
                                    "{}_{}_tilemap_image",
                                    block.block_x, block.block_y,
                                )),
                                buffer_view: Some(buffer_index),
                                mime_type: Some(json::image::MimeType("image/png".into())),
                                uri: None,
                                extensions: None,
                                extras: Default::default(),
                            });

                            let texture_index = Index::new(root.textures.len() as u32);
                            root.textures.push(json::Texture {
                                name: Some(format!(
                                    "{}_{}_tilemap_texture",
                                    block.block_x, block.block_y,
                                )),
                                sampler: Some(Index::new(0)),
                                source: image_index,
                                extensions: None,
                                extras: Default::default(),
                            });

                            let material_index = Index::<json::Material>::new(root.materials.len() as u32);
                            root.materials.push(json::Material {
                                name: Some(format!(
                                    "{}_{}_tilemap_material",
                                    block.block_x, block.block_y,
                                )),
                                alpha_cutoff: None,
                                alpha_mode: json::validation::Checked::Valid(AlphaMode::Opaque),
                                double_sided: false,
                                pbr_metallic_roughness: PbrMetallicRoughness {
                                    base_color_factor: PbrBaseColorFactor([
                                        1.0, 1.0, 1.0, 1.0,
                                    ]),
                                    base_color_texture: Some(json::texture::Info {
                                        index: texture_index,
                                        tex_coord: 0,
                                        extensions: None,
                                        extras: Default::default(),
                                    }),
                                    metallic_factor: StrengthFactor(0.0),
                                    roughness_factor: StrengthFactor(0.0),
                                    metallic_roughness_texture: None,
                                    extensions: None,
                                    extras: Default::default(),
                                },
                                normal_texture: None,
                                occlusion_texture: None,
                                emissive_texture: None,
                                emissive_factor: EmissiveFactor([0.0, 0.0, 0.0]),
                                extensions: None,
                                extras: Default::default(),
                            });
                            Some(material_index)
                        };

                        // HIM
                        let vertex_count = positions.len() as u32;
                        let vertex_data_start = binary_data.len() as u32;
                        for i in 0..positions.len() {
                            binary_data.put_f32_le(positions[i][0]);
                            binary_data.put_f32_le(positions[i][1]);
                            binary_data.put_f32_le(positions[i][2]);

                            binary_data.put_f32_le(normals[i][0]);
                            binary_data.put_f32_le(normals[i][1]);
                            binary_data.put_f32_le(normals[i][2]);

                            binary_data.put_f32_le(uvs[i][0]);
                            binary_data.put_f32_le(uvs[i][1]);
                        }
                        let vertex_data_length = binary_data.len() as u32 - vertex_data_start;
                        let vertex_data_stride = 3 * 4 + 3 * 4 + 2 * 4;

                        let index_data_start = binary_data.len() as u32;
                        for index in indices.iter() {
                            binary_data.put_u16_le(*index);
                        }
                        let index_data_length = binary_data.len() as u32 - index_data_start;

                        let vertex_data_buffer_view_index =
                            json::Index::new(root.buffer_views.len() as u32);
                        root.buffer_views.push(json::buffer::View {
                            name: Some(format!(
                                "{}_{}_VertexBufferView",
                                block.block_x, block.block_y
                            )),
                            buffer: json::Index::new(0),
                            byte_length: vertex_data_length,
                            byte_offset: Some(vertex_data_start),
                            byte_stride: Some(vertex_data_stride),
                            extensions: Default::default(),
                            extras: Default::default(),
                            target: Some(Valid(json::buffer::Target::ArrayBuffer)),
                        });

                        let index_data_buffer_view_index =
                            json::Index::new(root.buffer_views.len() as u32);
                        root.buffer_views.push(json::buffer::View {
                            name: Some(format!(
                                "{}_{}_IndexBufferView",
                                block.block_x, block.block_y
                            )),
                            buffer: json::Index::new(0),
                            byte_length: index_data_length,
                            byte_offset: Some(index_data_start),
                            byte_stride: None,
                            extensions: Default::default(),
                            extras: Default::default(),
                            target: Some(Valid(json::buffer::Target::ArrayBuffer)),
                        });

                        let indices_accessor_index = Index::new(root.accessors.len() as u32);
                        root.accessors.push(json::Accessor {
                            name: Some(format!("{}_{}_Indices", block.block_x, block.block_y)),
                            buffer_view: Some(index_data_buffer_view_index),
                            byte_offset: 0,
                            count: indices.len() as u32,
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

                        let mut attributes_map = HashMap::new();
                        let accessor_index: Index<json::Accessor> =
                            json::Index::new(root.accessors.len() as u32);
                        root.accessors.push(json::Accessor {
                            name: Some(format!(
                                "{}__{}_heightmap_verts",
                                block.block_x, block.block_y
                            )),
                            buffer_view: Some(vertex_data_buffer_view_index),
                            byte_offset: 0,
                            count: vertex_count,
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
                        attributes_map
                            .insert(Valid(json::mesh::Semantic::Positions), accessor_index);

                        let accessor_index: Index<json::Accessor> =
                            json::Index::new(root.accessors.len() as u32);
                        root.accessors.push(json::Accessor {
                            name: Some(format!(
                                "{}__{}_heightmap_normals",
                                block.block_x, block.block_y
                            )),
                            buffer_view: Some(vertex_data_buffer_view_index),
                            byte_offset: 4 * 3,
                            count: vertex_count,
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
                        attributes_map.insert(Valid(json::mesh::Semantic::Normals), accessor_index);

                        let accessor_index: Index<json::Accessor> =
                            json::Index::new(root.accessors.len() as u32);
                        root.accessors.push(json::Accessor {
                            name: Some(format!(
                                "{}__{}_heightmap_uvs",
                                block.block_x, block.block_y
                            )),
                            buffer_view: Some(vertex_data_buffer_view_index),
                            byte_offset: 3 * 4 + 3 * 4,
                            count: vertex_count,
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
                        attributes_map
                            .insert(Valid(json::mesh::Semantic::TexCoords(0)), accessor_index);

                        let heightmap_mesh = json::Index::new(root.meshes.len() as u32);
                        root.meshes.push(json::Mesh {
                            name: Some(format!(
                                "{}_{}_heightmap_mesh",
                                block.block_x, block.block_y
                            )),
                            extensions: Default::default(),
                            extras: Default::default(),
                            primitives: vec![json::mesh::Primitive {
                                attributes: attributes_map,
                                extensions: Default::default(),
                                extras: Default::default(),
                                indices: Some(indices_accessor_index),
                                material: heightmap_material,
                                mode: Valid(json::mesh::Mode::Triangles),
                                targets: None,
                            }],
                            weights: None,
                        });

                        let offset_x = (160.0 * block.block_x as f32) - 5200.0;
                        let offset_y = (160.0 * (65.0 - block.block_y as f32)) - 5200.0;
                        root.nodes.push(json::Node {
                            camera: None,
                            children: None,
                            extensions: Default::default(),
                            extras: Default::default(),
                            matrix: None,
                            mesh: Some(heightmap_mesh),
                            name: Some(format!("{}_{}_heightmap", block.block_x, block.block_y,)),
                            rotation: Some(UnitQuaternion::default()),
                            scale: Some([1.0, 1.0, 1.0]),
                            translation: Some([offset_x, 0.0, -offset_y]),
                            skin: None,
                            weights: None,
                        });
                    }

                    // Spawn all object nodes
                    for (object_instance_index, object_instance) in
                        block.ifo.objects.iter().enumerate()
                    {
                        let mut children = Vec::new();
                        let object_id = object_instance.object_id as usize;
                        let object = &deco.zsc.objects[object_id];

                        // Spawn a node for each object part
                        for (part_index, part) in object.parts.iter().enumerate() {
                            let mesh_data = deco.meshes.get(&part.mesh_id).expect("Missing mesh");
                            let mesh_index = root.meshes.len() as u32;
                            root.meshes.push(json::Mesh {
                                name: Some(format!(
                                    "{}_{}_obj{}_deco{}_part{}_mesh",
                                    block.block_x,
                                    block.block_y,
                                    object_instance_index,
                                    object_id,
                                    part_index
                                )),
                                extensions: Default::default(),
                                extras: Default::default(),
                                primitives: vec![json::mesh::Primitive {
                                    attributes: mesh_data.attributes.clone(),
                                    extensions: Default::default(),
                                    extras: Default::default(),
                                    indices: Some(mesh_data.indices),
                                    material: deco.materials.get(&part.material_id).copied(),
                                    mode: Valid(json::mesh::Mode::Triangles),
                                    targets: None,
                                }],
                                weights: None,
                            });

                            children.push(json::Index::new(root.nodes.len() as u32));
                            root.nodes.push(json::Node {
                                camera: None,
                                children: None,
                                extensions: Default::default(),
                                extras: Default::default(),
                                matrix: None,
                                mesh: Some(json::Index::new(mesh_index)),
                                name: Some(format!(
                                    "{}_{}_obj{}_deco{}_part{}",
                                    block.block_x,
                                    block.block_y,
                                    object_instance_index,
                                    object_id,
                                    part_index
                                )),
                                rotation: Some(convert_rotation(part.rotation)),
                                scale: Some([part.scale.x, part.scale.y, part.scale.z]),
                                translation: Some(convert_position(part.position)),
                                skin: None,
                                weights: None,
                            });
                        }

                        // Spawn a node for a object
                        root.nodes.push(json::Node {
                            camera: None,
                            children: Some(children),
                            extensions: Default::default(),
                            extras: Default::default(),
                            matrix: None,
                            mesh: None,
                            name: Some(format!(
                                "{}_{}_obj{}_deco{}",
                                block.block_x, block.block_y, object_instance_index, object_id
                            )),
                            rotation: Some(convert_rotation(object_instance.rotation)),
                            scale: Some([
                                object_instance.scale.x,
                                object_instance.scale.y,
                                object_instance.scale.z,
                            ]),
                            translation: Some(convert_position(object_instance.position)),
                            skin: None,
                            weights: None,
                        });
                    }

                    // Spawn a node for each building part
                    for (object_instance_index, object_instance) in
                        block.ifo.buildings.iter().enumerate()
                    {
                        let mut children = Vec::new();
                        let object_id = object_instance.object_id as usize;
                        let object = &cnst.zsc.objects[object_id];

                        // Spawn a node for each object part
                        for (part_index, part) in object.parts.iter().enumerate() {
                            let mesh_data = cnst.meshes.get(&part.mesh_id).expect("Missing mesh");
                            let mesh_index = root.meshes.len() as u32;
                            root.meshes.push(json::Mesh {
                                name: Some(format!(
                                    "{}_{}_obj{}_cnst{}_part{}_mesh",
                                    block.block_x,
                                    block.block_y,
                                    object_instance_index,
                                    object_id,
                                    part_index
                                )),
                                extensions: Default::default(),
                                extras: Default::default(),
                                primitives: vec![json::mesh::Primitive {
                                    attributes: mesh_data.attributes.clone(),
                                    extensions: Default::default(),
                                    extras: Default::default(),
                                    indices: Some(mesh_data.indices),
                                    material: cnst.materials.get(&part.material_id).copied(),
                                    mode: Valid(json::mesh::Mode::Triangles),
                                    targets: None,
                                }],
                                weights: None,
                            });

                            children.push(json::Index::new(root.nodes.len() as u32));
                            root.nodes.push(json::Node {
                                camera: None,
                                children: None,
                                extensions: Default::default(),
                                extras: Default::default(),
                                matrix: None,
                                mesh: Some(json::Index::new(mesh_index)),
                                name: Some(format!(
                                    "{}_{}_obj{}_cnst{}_part{}",
                                    block.block_x,
                                    block.block_y,
                                    object_instance_index,
                                    object_id,
                                    part_index
                                )),
                                rotation: Some(convert_rotation(part.rotation)),
                                scale: Some([part.scale.x, part.scale.y, part.scale.z]),
                                translation: Some(convert_position(part.position)),
                                skin: None,
                                weights: None,
                            });
                        }

                        // Spawn a node for building object
                        root.nodes.push(json::Node {
                            camera: None,
                            children: Some(children),
                            extensions: Default::default(),
                            extras: Default::default(),
                            matrix: None,
                            mesh: None,
                            name: Some(format!(
                                "{}_{}_obj{}_cnst{}",
                                block.block_x, block.block_y, object_instance_index, object_id
                            )),
                            rotation: Some(convert_rotation(object_instance.rotation)),
                            scale: Some([
                                object_instance.scale.x,
                                object_instance.scale.y,
                                object_instance.scale.z,
                            ]),
                            translation: Some(convert_position(object_instance.position)),
                            skin: None,
                            weights: None,
                        });
                    }
                }
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
