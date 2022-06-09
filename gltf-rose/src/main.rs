use std::path::PathBuf;

use gltf::mesh::util::{ReadColors, ReadIndices, ReadJoints, ReadTexCoords, ReadWeights};
use roselib::{
    files::{
        zms::{Vertex, VertexFormat},
        ZMS,
    },
    io::RoseFile,
    utils::Vector3,
};

fn main() {
    let matches = clap::Command::new("rose-gltf")
        .arg(clap::Arg::new("input-file").takes_value(true))
        .get_matches();

    let input_file = PathBuf::from(
        matches
            .value_of("input-file")
            .expect("No input file specified"),
    );

    let (document, buffers, _images) = gltf::import(&input_file).expect("Failed to read GLTF file");

    for (mesh_index, mesh) in document.meshes().enumerate() {
        let primitive = mesh
            .primitives()
            .next()
            .expect("Expected mesh to have 1 primitive");
        let mut zms = ZMS::new();
        let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

        if let Some(iter) = reader.read_positions() {
            zms.format |= VertexFormat::Position as i32;

            for position in iter {
                zms.vertices.push(Vertex {
                    position: Vector3 {
                        x: position[0],
                        y: -position[2],
                        z: position[1],
                    },
                    ..Default::default()
                });
            }

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
            zms.bounding_box.min = min_pos;
            zms.bounding_box.max = max_pos;
        }

        if let Some(iter) = reader.read_normals() {
            zms.format |= VertexFormat::Normal as i32;

            for (i, normal) in iter.enumerate() {
                zms.vertices[i].normal.x = normal[0];
                zms.vertices[i].normal.y = -normal[2];
                zms.vertices[i].normal.z = normal[1];
            }
        }

        if let Some(iter) = reader.read_tangents() {
            zms.format |= VertexFormat::Tangent as i32;

            for (i, tangent) in iter.enumerate() {
                zms.vertices[i].tangent.x = tangent[0];
                zms.vertices[i].tangent.y = -tangent[2];
                zms.vertices[i].tangent.z = tangent[1];
            }
        }

        if let Some(read_colors) = reader.read_colors(0) {
            zms.format |= VertexFormat::Color as i32;

            match read_colors {
                ReadColors::RgbU8(iter) => {
                    for (i, color) in iter.enumerate() {
                        zms.vertices[i].color.r = color[0] as f32 / 255.0;
                        zms.vertices[i].color.g = color[1] as f32 / 255.0;
                        zms.vertices[i].color.b = color[2] as f32 / 255.0;
                        zms.vertices[i].color.a = 1.0;
                    }
                }
                ReadColors::RgbaU8(iter) => {
                    for (i, color) in iter.enumerate() {
                        zms.vertices[i].color.r = color[0] as f32 / 255.0;
                        zms.vertices[i].color.g = color[1] as f32 / 255.0;
                        zms.vertices[i].color.b = color[2] as f32 / 255.0;
                        zms.vertices[i].color.a = color[3] as f32 / 255.0;
                    }
                }
                ReadColors::RgbU16(iter) => {
                    for (i, color) in iter.enumerate() {
                        zms.vertices[i].color.r = color[0] as f32 / 65535.0;
                        zms.vertices[i].color.g = color[1] as f32 / 65535.0;
                        zms.vertices[i].color.b = color[2] as f32 / 65535.0;
                        zms.vertices[i].color.a = 1.0;
                    }
                }
                ReadColors::RgbaU16(iter) => {
                    for (i, color) in iter.enumerate() {
                        zms.vertices[i].color.r = color[0] as f32 / 65535.0;
                        zms.vertices[i].color.g = color[1] as f32 / 65535.0;
                        zms.vertices[i].color.b = color[2] as f32 / 65535.0;
                        zms.vertices[i].color.a = color[3] as f32 / 65535.0;
                    }
                }
                ReadColors::RgbF32(iter) => {
                    for (i, color) in iter.enumerate() {
                        zms.vertices[i].color.r = color[0];
                        zms.vertices[i].color.g = color[1];
                        zms.vertices[i].color.b = color[2];
                        zms.vertices[i].color.a = 1.0;
                    }
                }
                ReadColors::RgbaF32(iter) => {
                    for (i, color) in iter.enumerate() {
                        zms.vertices[i].color.r = color[0];
                        zms.vertices[i].color.g = color[1];
                        zms.vertices[i].color.b = color[2];
                        zms.vertices[i].color.a = color[3];
                    }
                }
            }
        }

        if let Some(read_texcoords) = reader.read_tex_coords(0) {
            zms.format |= VertexFormat::UV1 as i32;

            match read_texcoords {
                ReadTexCoords::U8(iter) => {
                    for (i, uv1) in iter.enumerate() {
                        zms.vertices[i].uv1.x = uv1[0] as f32 / 255.0;
                        zms.vertices[i].uv1.y = uv1[1] as f32 / 255.0;
                    }
                }
                ReadTexCoords::U16(iter) => {
                    for (i, uv1) in iter.enumerate() {
                        zms.vertices[i].uv1.x = uv1[0] as f32 / 65535.0;
                        zms.vertices[i].uv1.y = uv1[1] as f32 / 65535.0;
                    }
                }
                ReadTexCoords::F32(iter) => {
                    for (i, uv1) in iter.enumerate() {
                        zms.vertices[i].uv1.x = uv1[0];
                        zms.vertices[i].uv1.y = uv1[1];
                    }
                }
            }
        }

        if let Some(read_texcoords) = reader.read_tex_coords(1) {
            zms.format |= VertexFormat::UV2 as i32;

            match read_texcoords {
                ReadTexCoords::U8(iter) => {
                    for (i, uv2) in iter.enumerate() {
                        zms.vertices[i].uv2.x = uv2[0] as f32 / 255.0;
                        zms.vertices[i].uv2.y = uv2[1] as f32 / 255.0;
                    }
                }
                ReadTexCoords::U16(iter) => {
                    for (i, uv2) in iter.enumerate() {
                        zms.vertices[i].uv2.x = uv2[0] as f32 / 65535.0;
                        zms.vertices[i].uv2.y = uv2[1] as f32 / 65535.0;
                    }
                }
                ReadTexCoords::F32(iter) => {
                    for (i, uv2) in iter.enumerate() {
                        zms.vertices[i].uv2.x = uv2[0];
                        zms.vertices[i].uv2.y = uv2[1];
                    }
                }
            }
        }

        if let Some(read_texcoords) = reader.read_tex_coords(2) {
            zms.format |= VertexFormat::UV3 as i32;

            match read_texcoords {
                ReadTexCoords::U8(iter) => {
                    for (i, uv3) in iter.enumerate() {
                        zms.vertices[i].uv3.x = uv3[0] as f32 / 255.0;
                        zms.vertices[i].uv3.y = uv3[1] as f32 / 255.0;
                    }
                }
                ReadTexCoords::U16(iter) => {
                    for (i, uv3) in iter.enumerate() {
                        zms.vertices[i].uv3.x = uv3[0] as f32 / 65535.0;
                        zms.vertices[i].uv3.y = uv3[1] as f32 / 65535.0;
                    }
                }
                ReadTexCoords::F32(iter) => {
                    for (i, uv3) in iter.enumerate() {
                        zms.vertices[i].uv3.x = uv3[0];
                        zms.vertices[i].uv3.y = uv3[1];
                    }
                }
            }
        }

        if let Some(read_texcoords) = reader.read_tex_coords(3) {
            zms.format |= VertexFormat::UV4 as i32;

            match read_texcoords {
                ReadTexCoords::U8(iter) => {
                    for (i, uv4) in iter.enumerate() {
                        zms.vertices[i].uv4.x = uv4[0] as f32 / 255.0;
                        zms.vertices[i].uv4.y = uv4[1] as f32 / 255.0;
                    }
                }
                ReadTexCoords::U16(iter) => {
                    for (i, uv4) in iter.enumerate() {
                        zms.vertices[i].uv4.x = uv4[0] as f32 / 65535.0;
                        zms.vertices[i].uv4.y = uv4[1] as f32 / 65535.0;
                    }
                }
                ReadTexCoords::F32(iter) => {
                    for (i, uv4) in iter.enumerate() {
                        zms.vertices[i].uv4.x = uv4[0];
                        zms.vertices[i].uv4.y = uv4[1];
                    }
                }
            }
        }

        if let Some(read_joints) = reader.read_joints(0) {
            zms.format |= VertexFormat::BoneIndex as i32;

            match read_joints {
                ReadJoints::U8(iter) => {
                    for (i, joints) in iter.enumerate() {
                        zms.vertices[i].bone_indices.x = joints[0] as i16;
                        zms.vertices[i].bone_indices.y = joints[1] as i16;
                        zms.vertices[i].bone_indices.z = joints[2] as i16;
                        zms.vertices[i].bone_indices.w = joints[3] as i16;
                    }
                }
                ReadJoints::U16(iter) => {
                    for (i, joints) in iter.enumerate() {
                        zms.vertices[i].bone_indices.x = joints[0] as i16;
                        zms.vertices[i].bone_indices.y = joints[1] as i16;
                        zms.vertices[i].bone_indices.z = joints[2] as i16;
                        zms.vertices[i].bone_indices.w = joints[3] as i16;
                    }
                }
            }

            let mut max_bone_index = 0;
            for vertex in zms.vertices.iter() {
                max_bone_index = max_bone_index.max(vertex.bone_indices.x);
                max_bone_index = max_bone_index.max(vertex.bone_indices.y);
                max_bone_index = max_bone_index.max(vertex.bone_indices.z);
                max_bone_index = max_bone_index.max(vertex.bone_indices.w);
            }

            for i in 0..=max_bone_index {
                zms.bones.push(i);
            }
        }

        if let Some(read_weights) = reader.read_weights(0) {
            zms.format |= VertexFormat::BoneWeight as i32;

            match read_weights {
                ReadWeights::U8(iter) => {
                    for (i, weights) in iter.enumerate() {
                        zms.vertices[i].bone_weights.x = weights[0] as f32 / 255.0;
                        zms.vertices[i].bone_weights.y = weights[1] as f32 / 255.0;
                        zms.vertices[i].bone_weights.z = weights[2] as f32 / 255.0;
                        zms.vertices[i].bone_weights.w = weights[3] as f32 / 255.0;
                    }
                }
                ReadWeights::U16(iter) => {
                    for (i, weights) in iter.enumerate() {
                        zms.vertices[i].bone_weights.x = weights[0] as f32 / 65535.0;
                        zms.vertices[i].bone_weights.y = weights[1] as f32 / 65535.0;
                        zms.vertices[i].bone_weights.z = weights[2] as f32 / 65535.0;
                        zms.vertices[i].bone_weights.w = weights[3] as f32 / 65535.0;
                    }
                }
                ReadWeights::F32(iter) => {
                    for (i, weights) in iter.enumerate() {
                        zms.vertices[i].bone_weights.x = weights[0];
                        zms.vertices[i].bone_weights.y = weights[1];
                        zms.vertices[i].bone_weights.z = weights[2];
                        zms.vertices[i].bone_weights.w = weights[3];
                    }
                }
            }
        }

        if let Some(read_indices) = reader.read_indices() {
            let mut indices = Vec::new();

            match read_indices {
                ReadIndices::U8(iter) => {
                    for i in iter {
                        indices.push(i as i16);
                    }
                }
                ReadIndices::U16(iter) => {
                    for i in iter {
                        indices.push(i as i16);
                    }
                }
                ReadIndices::U32(iter) => {
                    for i in iter {
                        indices.push(i as i16);
                    }
                }
            }

            for triangle in indices.chunks_exact(3) {
                zms.indices.push(Vector3 {
                    x: triangle[0],
                    y: triangle[1],
                    z: triangle[2],
                });
            }
        }

        let out_path = PathBuf::from(
            mesh.name()
                .map(|x| x.to_string())
                .unwrap_or(format!("mesh_{}.zms", mesh_index)),
        );
        zms.write_to_path(&out_path)
            .expect("Failed to write output file");
    }
}
