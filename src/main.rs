use bytes::{BufMut, BytesMut};
use glam::{Mat4, Quat, Vec3};
use gltf_json as json;
use json::{scene::UnitQuaternion, validation::Checked::Valid};
use rose_file_readers::{RoseFile, VfsFile, ZmdFile, ZmsFile};
use std::{borrow::Cow, collections::HashMap, path::PathBuf};

fn load_mesh(root: &mut json::Root, binary_data: &mut BytesMut, name: &str, zms: &ZmsFile) -> u32 {
    let mut attributes_map = HashMap::new();
    let mut vertex_data_stride = 0;
    let vertex_count = zms.position.len();
    let vertex_data_buffer_view_index = root.buffer_views.len() as u32;
    let index_data_buffer_view_index = vertex_data_buffer_view_index + 1;

    if !zms.position.is_empty() {
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
            min: None,
            max: None,
            normalized: false,
            sparse: None,
        });
        attributes_map.insert(
            Valid(json::mesh::Semantic::Positions),
            json::Index::new(accessor_index),
        );
        vertex_data_stride += 4 * 3;
    }

    if !zms.normal.is_empty() {
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

    if !zms.tangent.is_empty() {
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

    if !zms.color.is_empty() {
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

    if !zms.uv1.is_empty() {
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

    if !zms.uv2.is_empty() {
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

    if !zms.uv3.is_empty() {
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

    if !zms.uv4.is_empty() {
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

    if !zms.bone_weights.is_empty() {
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
    }

    if !zms.bone_indices.is_empty() {
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
    for i in 0..zms.position.len() {
        if !zms.position.is_empty() {
            binary_data.put_f32_le(zms.position[i][0]);
            binary_data.put_f32_le(zms.position[i][2]);
            binary_data.put_f32_le(-zms.position[i][1]);
        }

        if !zms.normal.is_empty() {
            binary_data.put_f32_le(zms.normal[i][0]);
            binary_data.put_f32_le(zms.normal[i][2]);
            binary_data.put_f32_le(-zms.normal[i][1]);
        }

        if !zms.tangent.is_empty() {
            binary_data.put_f32_le(zms.tangent[i][0]);
            binary_data.put_f32_le(zms.tangent[i][2]);
            binary_data.put_f32_le(-zms.tangent[i][1]);
        }

        if !zms.color.is_empty() {
            binary_data.put_f32_le(zms.color[i][0]);
            binary_data.put_f32_le(zms.color[i][1]);
            binary_data.put_f32_le(zms.color[i][2]);
            binary_data.put_f32_le(zms.color[i][3]);
        }

        if !zms.uv1.is_empty() {
            binary_data.put_f32_le(zms.uv1[i][0]);
            binary_data.put_f32_le(zms.uv1[i][1]);
        }

        if !zms.uv2.is_empty() {
            binary_data.put_f32_le(zms.uv2[i][0]);
            binary_data.put_f32_le(zms.uv2[i][1]);
        }

        if !zms.uv3.is_empty() {
            binary_data.put_f32_le(zms.uv3[i][0]);
            binary_data.put_f32_le(zms.uv3[i][1]);
        }

        if !zms.uv4.is_empty() {
            binary_data.put_f32_le(zms.uv4[i][0]);
            binary_data.put_f32_le(zms.uv4[i][1]);
        }

        if !zms.bone_weights.is_empty() {
            binary_data.put_f32_le(zms.bone_weights[i][0]);
            binary_data.put_f32_le(zms.bone_weights[i][1]);
            binary_data.put_f32_le(zms.bone_weights[i][2]);
            binary_data.put_f32_le(zms.bone_weights[i][3]);
        }

        if !zms.bone_indices.is_empty() {
            binary_data.put_u16_le(zms.bone_indices[i][0]);
            binary_data.put_u16_le(zms.bone_indices[i][1]);
            binary_data.put_u16_le(zms.bone_indices[i][2]);
            binary_data.put_u16_le(zms.bone_indices[i][3]);
        }
    }
    let vertex_data_length = binary_data.len() as u32 - vertex_data_start;

    let index_data_start = binary_data.len() as u32;
    let index_data_stride = 2;
    for i in 0..zms.indices.len() {
        binary_data.put_u16_le(zms.indices[i]);
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
        count: zms.indices.len() as u32,
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

fn transform_children(skeleton: &ZmdFile, bone_transforms: &mut Vec<Mat4>, bone_index: usize) {
    for (child_id, child_bone) in skeleton.bones.iter().enumerate() {
        if child_id == bone_index || child_bone.parent as usize != bone_index {
            continue;
        }

        bone_transforms[child_id] = bone_transforms[bone_index] * bone_transforms[child_id];
        transform_children(skeleton, bone_transforms, child_id);
    }
}

fn load_skeleton(
    root: &mut json::Root,
    binary_data: &mut BytesMut,
    name: &str,
    zmd: &ZmdFile,
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
        let file_data = std::fs::read(&file_path).expect("Failed to read input file");
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
                let zmd = <ZmdFile as RoseFile>::read(
                    (&VfsFile::Buffer(file_data)).into(),
                    &Default::default(),
                )
                .expect("Failed to parse ZMD");

                skin_index = Some(load_skeleton(&mut root, &mut binary_data, &file_name, &zmd));
            }
            "zms" => {
                let zms = <ZmsFile as RoseFile>::read(
                    (&VfsFile::Buffer(file_data)).into(),
                    &Default::default(),
                )
                .expect("Failed to parse ZMS");

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
                    skin: if zms.bone_indices.is_empty() {
                        None
                    } else {
                        skin_index
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
