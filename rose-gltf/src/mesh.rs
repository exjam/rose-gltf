use bytes::BytesMut;
use glam::{Vec2, Vec3, Vec4};
use gltf_json::{mesh, validation::Checked};
use roselib::files::ZMS;

use crate::mesh_builder::{MeshBuilder, MeshData};

pub fn load_mesh_data(
    root: &mut gltf_json::Root,
    binary_data: &mut BytesMut,
    name: &str,
    zms: &ZMS,
) -> MeshData {
    let mut mesh_builder = MeshBuilder::new();
    mesh_builder.add_indices(
        zms.indices
            .iter()
            .flat_map(|triangle| [triangle.x as u16, triangle.y as u16, triangle.z as u16])
            .collect(),
    );

    mesh_builder.add_positions(
        zms.vertices
            .iter()
            .map(|vertex| Vec3::new(vertex.position.x, vertex.position.z, -vertex.position.y))
            .collect(),
    );

    if zms.normals_enabled() {
        mesh_builder.add_normals(
            zms.vertices
                .iter()
                .map(|vertex| Vec3::new(vertex.normal.x, vertex.normal.z, -vertex.normal.y))
                .collect(),
        );
    }

    if zms.tangents_enabled() {
        mesh_builder.add_tangents(
            zms.vertices
                .iter()
                .map(|vertex| Vec3::new(vertex.tangent.x, vertex.tangent.z, -vertex.tangent.y))
                .collect(),
        );
    }

    if zms.colors_enabled() {
        mesh_builder.add_color(
            zms.vertices
                .iter()
                .map(|vertex| {
                    Vec4::new(
                        vertex.color.r,
                        vertex.color.g,
                        vertex.color.b,
                        vertex.color.a,
                    )
                })
                .collect(),
        );
    }

    if zms.uv1_enabled() {
        mesh_builder.add_uv0(
            zms.vertices
                .iter()
                .map(|vertex| Vec2::new(vertex.uv1.x, vertex.uv1.y))
                .collect(),
        );
    }

    if zms.uv2_enabled() {
        mesh_builder.add_uv1(
            zms.vertices
                .iter()
                .map(|vertex| Vec2::new(vertex.uv2.x, vertex.uv2.y))
                .collect(),
        );
    }

    if zms.uv3_enabled() {
        mesh_builder.add_uv2(
            zms.vertices
                .iter()
                .map(|vertex| Vec2::new(vertex.uv3.x, vertex.uv3.y))
                .collect(),
        );
    }

    if zms.uv4_enabled() {
        mesh_builder.add_uv3(
            zms.vertices
                .iter()
                .map(|vertex| Vec2::new(vertex.uv4.x, vertex.uv4.y))
                .collect(),
        );
    }

    if zms.bones_enabled() {
        mesh_builder.add_bone_weight(
            zms.vertices
                .iter()
                .map(|vertex| {
                    Vec4::new(
                        vertex.bone_weights.x,
                        vertex.bone_weights.y,
                        vertex.bone_weights.z,
                        vertex.bone_weights.w,
                    )
                })
                .collect(),
        );
        mesh_builder.add_bone_index(
            zms.vertices
                .iter()
                .map(|vertex| {
                    [
                        if vertex.bone_weights.x == 0.0 {
                            0
                        } else {
                            zms.bones[vertex.bone_indices.x as usize] as u16
                        },
                        if vertex.bone_weights.y == 0.0 {
                            0
                        } else {
                            zms.bones[vertex.bone_indices.y as usize] as u16
                        },
                        if vertex.bone_weights.z == 0.0 {
                            0
                        } else {
                            zms.bones[vertex.bone_indices.z as usize] as u16
                        },
                        if vertex.bone_weights.w == 0.0 {
                            0
                        } else {
                            zms.bones[vertex.bone_indices.w as usize] as u16
                        },
                    ]
                })
                .collect(),
        )
    }

    mesh_builder.build(root, binary_data, name)
}

pub fn load_mesh(
    root: &mut gltf_json::Root,
    binary_data: &mut BytesMut,
    name: &str,
    zms: &ZMS,
) -> u32 {
    let mesh_data = load_mesh_data(root, binary_data, name, zms);
    let mesh_index = root.meshes.len() as u32;
    root.meshes.push(mesh::Mesh {
        name: Some(name.into()),
        extensions: Default::default(),
        extras: Default::default(),
        primitives: vec![mesh::Primitive {
            attributes: mesh_data.attributes,
            extensions: Default::default(),
            extras: Default::default(),
            indices: Some(mesh_data.indices),
            material: None,
            mode: Checked::Valid(mesh::Mode::Triangles),
            targets: None,
        }],
        weights: None,
    });

    mesh_index
}
