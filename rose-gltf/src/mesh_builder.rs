use std::collections::{BTreeMap, HashMap};

use bytes::{BufMut, BytesMut};
use glam::{Vec2, Vec3, Vec4};
use gltf_json::{accessor, buffer, mesh::Semantic, validation::Checked, Index};
use serde_json::json;

use crate::pad_align;

#[derive(Default)]
pub struct MeshBuilder {
    position: Vec<Vec3>,
    position_min: Vec3,
    position_max: Vec3,
    indices: Vec<u16>,
    normal: Vec<Vec3>,
    tangent: Vec<Vec3>,
    uv0: Vec<Vec2>,
    uv1: Vec<Vec2>,
    uv2: Vec<Vec2>,
    uv3: Vec<Vec2>,
    color: Vec<Vec4>,
    bone_weight: Vec<Vec4>,
    bone_index: Vec<[u16; 4]>,
}

#[derive(Clone)]
pub struct MeshData {
    pub attributes: BTreeMap<Checked<Semantic>, Index<accessor::Accessor>>,
    pub indices: Index<accessor::Accessor>,
}

impl MeshBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_positions(&mut self, position: Vec<Vec3>) {
        self.position = position;

        // Calculate min / max
        self.position_min = self.position[0];
        self.position_max = self.position[0];
        for pos in self.position.iter() {
            self.position_min = self.position_min.min(*pos);
            self.position_max = self.position_max.max(*pos);
        }
    }

    pub fn add_indices(&mut self, indices: Vec<u16>) {
        self.indices = indices;
    }

    pub fn add_normals(&mut self, normals: Vec<Vec3>) {
        self.normal = normals;
        for normal in self.normal.iter_mut() {
            *normal = normal.normalize();
        }
    }

    pub fn add_tangents(&mut self, tangents: Vec<Vec3>) {
        self.tangent = tangents;
        for tangent in self.tangent.iter_mut() {
            *tangent = tangent.normalize();
        }
    }

    pub fn add_uv0(&mut self, uv0: Vec<Vec2>) {
        self.uv0 = uv0;
    }

    pub fn add_uv1(&mut self, uv1: Vec<Vec2>) {
        self.uv1 = uv1;
    }

    pub fn add_uv2(&mut self, uv2: Vec<Vec2>) {
        self.uv2 = uv2;
    }

    pub fn add_uv3(&mut self, uv3: Vec<Vec2>) {
        self.uv3 = uv3;
    }

    pub fn add_color(&mut self, color: Vec<Vec4>) {
        self.color = color;
    }

    pub fn add_bone_weight(&mut self, bone_weight: Vec<Vec4>) {
        self.bone_weight = bone_weight;
    }

    pub fn add_bone_index(&mut self, bone_index: Vec<[u16; 4]>) {
        self.bone_index = bone_index;
    }

    pub fn build(
        self,
        root: &mut gltf_json::Root,
        binary_data: &mut BytesMut,
        name: &str,
    ) -> MeshData {
        let mut attributes = BTreeMap::new();
        let mut vertex_data_stride = 0;
        let vertex_count = self.position.len();
        let vertex_buffer_view = Index::new(root.buffer_views.len() as u32);
        let index_buffer_view = Index::new(root.buffer_views.len() as u32 + 1);

        let accesor = Index::new(root.accessors.len() as u32);
        root.accessors.push(accessor::Accessor {
            name: Some(format!("{}_position", name)),
            buffer_view: Some(vertex_buffer_view),
            byte_offset: Some(vertex_data_stride),
            count: vertex_count as u32,
            component_type: Checked::Valid(accessor::GenericComponentType(
                accessor::ComponentType::F32,
            )),
            extensions: Default::default(),
            extras: Default::default(),
            type_: Checked::Valid(accessor::Type::Vec3),
            min: Some(json!(self.position_min.to_array())),
            max: Some(json!(self.position_max.to_array())),
            normalized: false,
            sparse: None,
        });
        attributes.insert(Checked::Valid(Semantic::Positions), accesor);
        vertex_data_stride += 4 * 3;

        if !self.normal.is_empty() {
            let accessor = Index::new(root.accessors.len() as u32);
            root.accessors.push(accessor::Accessor {
                name: Some(format!("{}_normal", name)),
                buffer_view: Some(vertex_buffer_view),
                byte_offset: Some(vertex_data_stride),
                count: vertex_count as u32,
                component_type: Checked::Valid(accessor::GenericComponentType(
                    accessor::ComponentType::F32,
                )),
                extensions: Default::default(),
                extras: Default::default(),
                type_: Checked::Valid(accessor::Type::Vec3),
                min: None,
                max: None,
                normalized: false,
                sparse: None,
            });
            attributes.insert(Checked::Valid(Semantic::Normals), accessor);
            vertex_data_stride += 4 * 3;
        }

        if !self.tangent.is_empty() {
            let accessor = Index::new(root.accessors.len() as u32);
            root.accessors.push(accessor::Accessor {
                name: Some(format!("{}_tangent", name)),
                buffer_view: Some(vertex_buffer_view),
                byte_offset: Some(vertex_data_stride),
                count: vertex_count as u32,
                component_type: Checked::Valid(accessor::GenericComponentType(
                    accessor::ComponentType::F32,
                )),
                extensions: Default::default(),
                extras: Default::default(),
                type_: Checked::Valid(accessor::Type::Vec3),
                min: None,
                max: None,
                normalized: false,
                sparse: None,
            });
            attributes.insert(Checked::Valid(Semantic::Tangents), accessor);
            vertex_data_stride += 4 * 3;
        }

        if !self.color.is_empty() {
            let accessor = Index::new(root.accessors.len() as u32);
            root.accessors.push(accessor::Accessor {
                name: Some(format!("{}_color", name)),
                buffer_view: Some(vertex_buffer_view),
                byte_offset: Some(vertex_data_stride),
                count: vertex_count as u32,
                component_type: Checked::Valid(accessor::GenericComponentType(
                    accessor::ComponentType::F32,
                )),
                extensions: Default::default(),
                extras: Default::default(),
                type_: Checked::Valid(accessor::Type::Vec4),
                min: None,
                max: None,
                normalized: false,
                sparse: None,
            });
            attributes.insert(Checked::Valid(Semantic::Colors(0)), accessor);
            vertex_data_stride += 4 * 4;
        }

        if !self.uv0.is_empty() {
            let accessor = Index::new(root.accessors.len() as u32);
            root.accessors.push(accessor::Accessor {
                name: Some(format!("{}_uv0", name)),
                buffer_view: Some(vertex_buffer_view),
                byte_offset: Some(vertex_data_stride),
                count: vertex_count as u32,
                component_type: Checked::Valid(accessor::GenericComponentType(
                    accessor::ComponentType::F32,
                )),
                extensions: Default::default(),
                extras: Default::default(),
                type_: Checked::Valid(accessor::Type::Vec2),
                min: None,
                max: None,
                normalized: false,
                sparse: None,
            });
            attributes.insert(Checked::Valid(Semantic::TexCoords(0)), accessor);
            vertex_data_stride += 4 * 2;
        }

        if !self.uv1.is_empty() {
            let accessor = Index::new(root.accessors.len() as u32);
            root.accessors.push(accessor::Accessor {
                name: Some(format!("{}_uv1", name)),
                buffer_view: Some(vertex_buffer_view),
                byte_offset: Some(vertex_data_stride),
                count: vertex_count as u32,
                component_type: Checked::Valid(accessor::GenericComponentType(
                    accessor::ComponentType::F32,
                )),
                extensions: Default::default(),
                extras: Default::default(),
                type_: Checked::Valid(accessor::Type::Vec2),
                min: None,
                max: None,
                normalized: false,
                sparse: None,
            });
            attributes.insert(Checked::Valid(Semantic::TexCoords(1)), accessor);
            vertex_data_stride += 4 * 2;
        }

        if !self.uv2.is_empty() {
            let accessor = Index::new(root.accessors.len() as u32);
            root.accessors.push(accessor::Accessor {
                name: Some(format!("{}_uv2", name)),
                buffer_view: Some(vertex_buffer_view),
                byte_offset: Some(vertex_data_stride),
                count: vertex_count as u32,
                component_type: Checked::Valid(accessor::GenericComponentType(
                    accessor::ComponentType::F32,
                )),
                extensions: Default::default(),
                extras: Default::default(),
                type_: Checked::Valid(accessor::Type::Vec2),
                min: None,
                max: None,
                normalized: false,
                sparse: None,
            });
            attributes.insert(Checked::Valid(Semantic::TexCoords(2)), accessor);
            vertex_data_stride += 4 * 2;
        }

        if !self.uv3.is_empty() {
            let accessor = Index::new(root.accessors.len() as u32);
            root.accessors.push(accessor::Accessor {
                name: Some(format!("{}_uv3", name)),
                buffer_view: Some(vertex_buffer_view),
                byte_offset: Some(vertex_data_stride),
                count: vertex_count as u32,
                component_type: Checked::Valid(accessor::GenericComponentType(
                    accessor::ComponentType::F32,
                )),
                extensions: Default::default(),
                extras: Default::default(),
                type_: Checked::Valid(accessor::Type::Vec2),
                min: None,
                max: None,
                normalized: false,
                sparse: None,
            });
            attributes.insert(Checked::Valid(Semantic::TexCoords(3)), accessor);
            vertex_data_stride += 4 * 2;
        }

        if !self.bone_weight.is_empty() {
            let accessor = Index::new(root.accessors.len() as u32);
            root.accessors.push(accessor::Accessor {
                name: Some(format!("{}_boneweight", name)),
                buffer_view: Some(vertex_buffer_view),
                byte_offset: Some(vertex_data_stride),
                count: vertex_count as u32,
                component_type: Checked::Valid(accessor::GenericComponentType(
                    accessor::ComponentType::F32,
                )),
                extensions: Default::default(),
                extras: Default::default(),
                type_: Checked::Valid(accessor::Type::Vec4),
                min: None,
                max: None,
                normalized: false,
                sparse: None,
            });
            attributes.insert(Checked::Valid(Semantic::Weights(0)), accessor);
            vertex_data_stride += 4 * 4;
        }

        if !self.bone_index.is_empty() {
            let accessor = Index::new(root.accessors.len() as u32);
            root.accessors.push(accessor::Accessor {
                name: Some(format!("{}_boneindex", name)),
                buffer_view: Some(vertex_buffer_view),
                byte_offset: Some(vertex_data_stride),
                count: vertex_count as u32,
                component_type: Checked::Valid(accessor::GenericComponentType(
                    accessor::ComponentType::U16,
                )),
                extensions: Default::default(),
                extras: Default::default(),
                type_: Checked::Valid(accessor::Type::Vec4),
                min: None,
                max: None,
                normalized: false,
                sparse: None,
            });
            attributes.insert(Checked::Valid(Semantic::Joints(0)), accessor);
            vertex_data_stride += 4 * 2;
        }

        pad_align(binary_data);
        let vertex_data_start = binary_data.len() as u32;
        for i in 0..vertex_count {
            binary_data.put_f32_le(self.position[i].x);
            binary_data.put_f32_le(self.position[i].y);
            binary_data.put_f32_le(self.position[i].z);

            if !self.normal.is_empty() {
                binary_data.put_f32_le(self.normal[i].x);
                binary_data.put_f32_le(self.normal[i].y);
                binary_data.put_f32_le(self.normal[i].z);
            }

            if !self.tangent.is_empty() {
                binary_data.put_f32_le(self.tangent[i].x);
                binary_data.put_f32_le(self.tangent[i].y);
                binary_data.put_f32_le(self.tangent[i].z);
            }

            if !self.color.is_empty() {
                binary_data.put_f32_le(self.color[i].x);
                binary_data.put_f32_le(self.color[i].y);
                binary_data.put_f32_le(self.color[i].z);
                binary_data.put_f32_le(self.color[i].w);
            }

            if !self.uv0.is_empty() {
                binary_data.put_f32_le(self.uv0[i].x);
                binary_data.put_f32_le(self.uv0[i].y);
            }

            if !self.uv1.is_empty() {
                binary_data.put_f32_le(self.uv1[i].x);
                binary_data.put_f32_le(self.uv1[i].y);
            }

            if !self.uv2.is_empty() {
                binary_data.put_f32_le(self.uv2[i].x);
                binary_data.put_f32_le(self.uv2[i].y);
            }

            if !self.uv3.is_empty() {
                binary_data.put_f32_le(self.uv3[i].x);
                binary_data.put_f32_le(self.uv3[i].y);
            }

            if !self.bone_weight.is_empty() {
                binary_data.put_f32_le(self.bone_weight[i].x);
                binary_data.put_f32_le(self.bone_weight[i].y);
                binary_data.put_f32_le(self.bone_weight[i].z);
                binary_data.put_f32_le(self.bone_weight[i].w);
            }

            if !self.bone_index.is_empty() {
                binary_data.put_u16_le(self.bone_index[i][0]);
                binary_data.put_u16_le(self.bone_index[i][1]);
                binary_data.put_u16_le(self.bone_index[i][2]);
                binary_data.put_u16_le(self.bone_index[i][3]);
            }
        }
        let vertex_data_length = binary_data.len() as u32 - vertex_data_start;

        pad_align(binary_data);

        let index_data_start = binary_data.len() as u32;
        for index in self.indices.iter() {
            binary_data.put_u16_le(*index);
        }
        let index_data_length = binary_data.len() as u32 - index_data_start;
        pad_align(binary_data);

        root.buffer_views.push(buffer::View {
            name: Some(format!("{}_vbuffer", name)),
            buffer: Index::new(0),
            byte_length: vertex_data_length,
            byte_offset: Some(vertex_data_start),
            byte_stride: Some(vertex_data_stride),
            extensions: Default::default(),
            extras: Default::default(),
            target: Some(Checked::Valid(buffer::Target::ArrayBuffer)),
        });

        root.buffer_views.push(buffer::View {
            name: Some(format!("{}_ibuffer", name)),
            buffer: Index::new(0),
            byte_length: index_data_length,
            byte_offset: Some(index_data_start),
            byte_stride: None,
            extensions: Default::default(),
            extras: Default::default(),
            target: Some(Checked::Valid(buffer::Target::ElementArrayBuffer)),
        });

        let index_buffer_accessor = Index::new(root.accessors.len() as u32);
        root.accessors.push(accessor::Accessor {
            name: Some(format!("{}_Indices", name)),
            buffer_view: Some(index_buffer_view),
            byte_offset: Some(0),
            count: self.indices.len() as u32,
            component_type: Checked::Valid(accessor::GenericComponentType(
                accessor::ComponentType::U16,
            )),
            extensions: Default::default(),
            extras: Default::default(),
            type_: Checked::Valid(accessor::Type::Scalar),
            min: None,
            max: None,
            normalized: false,
            sparse: None,
        });

        MeshData {
            attributes,
            indices: index_buffer_accessor,
        }
    }
}
