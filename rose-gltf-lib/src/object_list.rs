use std::{collections::HashMap, io::Cursor, path::Path};

use anyhow::Context;
use bytes::{BufMut, BytesMut};
use gltf_json::{
    buffer, material, texture,
    validation::{Checked, USize64},
    Index,
};
use image::{DynamicImage, ImageBuffer, Rgba};
use rose_file_lib::{
    files::{zsc, ZMS, ZSC},
    io::RoseFile,
};

use crate::{mesh::load_mesh_data, mesh_builder::MeshData, pad_align};

pub struct ObjectList {
    pub zsc: ZSC,
    pub materials: HashMap<zsc::ModelMaterial, Index<material::Material>>,
    pub meshes: HashMap<String, MeshData>,
    pub sampler: Index<texture::Sampler>,
}

impl ObjectList {
    pub fn new(zsc: ZSC, sampler: Index<texture::Sampler>) -> Self {
        Self {
            materials: HashMap::new(),
            meshes: HashMap::new(),
            zsc,
            sampler,
        }
    }

    pub fn load_object(
        &mut self,
        name_prefix: &str,
        object_id: usize,
        root: &mut gltf_json::Root,
        binary_data: &mut BytesMut,
        assets_path: &Path,
    ) -> anyhow::Result<()> {
        let object = self
            .zsc
            .models
            .get(object_id)
            .with_context(|| format!("Invalid object id: {}", object_id))?;

        for part in object
            .as_ref()
            .map_or([].iter(), |object| object.parts.iter())
        {
            if let Some(material) = part.material.as_ref() {
                if let Some(material_data) =
                    self.load_material(name_prefix, material, root, binary_data, assets_path)
                {
                    self.materials.insert(material.clone(), material_data);
                }
            }

            if let Some(mesh_data) =
                self.load_mesh(name_prefix, &part.mesh_path, root, binary_data, assets_path)
            {
                self.meshes.insert(part.mesh_path.clone(), mesh_data);
            }
        }

        Ok(())
    }

    pub fn load_mesh(
        &self,
        name_prefix: &str,
        mesh_path: &str,
        root: &mut gltf_json::Root,
        binary_data: &mut BytesMut,
        assets_path: &Path,
    ) -> Option<MeshData> {
        if self.meshes.contains_key(mesh_path) {
            // Already loaded
            return None;
        }

        let zms = ZMS::from_path(&assets_path.join(mesh_path)).expect("Failed to load ZMS");
        let mesh_id = self.meshes.len();
        Some(load_mesh_data(
            root,
            binary_data,
            &format!("{}_mesh_{}", name_prefix, mesh_id),
            &zms,
            true, // Seems like lots of objects have busted normals
        ))
    }

    pub fn load_material(
        &self,
        name_prefix: &str,
        material: &zsc::ModelMaterial,
        root: &mut gltf_json::Root,
        binary_data: &mut BytesMut,
        assets_path: &Path,
    ) -> Option<Index<material::Material>> {
        if self.materials.contains_key(material) {
            // Already loaded
            return None;
        }

        let material_id = self.materials.len();
        let img = match image::open(assets_path.join(&material.path)) {
            Ok(img) => img,
            Err(error) => {
                println!("Failed to read {} with error {}", material.path, error);
                DynamicImage::ImageRgba8(ImageBuffer::from_pixel(
                    4,
                    4,
                    Rgba([255u8, 1u8, 255u8, 0u8]),
                ))
            }
        };
        let mut png_buffer: Vec<u8> = Vec::new();
        img.write_to(&mut Cursor::new(&mut png_buffer), image::ImageFormat::Png)
            .expect("Failed to write PNG");

        pad_align(binary_data);
        let texture_data_start = binary_data.len();
        binary_data.put_slice(&png_buffer);
        pad_align(binary_data);

        let buffer_index = Index::new(root.buffer_views.len() as u32);
        root.buffer_views.push(buffer::View {
            name: Some(format!(
                "{}_material_{}_image_buffer",
                name_prefix, material_id
            )),
            buffer: Index::new(0),
            byte_length: USize64::from(png_buffer.len()),
            byte_offset: Some(USize64::from(texture_data_start)),
            byte_stride: None,
            extensions: Default::default(),
            extras: Default::default(),
            target: None,
        });

        let image_index = Index::new(root.images.len() as u32);
        root.images.push(gltf_json::Image {
            name: Some(format!("{}_material_{}_image", name_prefix, material_id)),
            buffer_view: Some(buffer_index),
            mime_type: Some(gltf_json::image::MimeType("image/png".into())),
            uri: None,
            extensions: None,
            extras: Default::default(),
        });

        let texture_index = Index::new(root.textures.len() as u32);
        root.textures.push(texture::Texture {
            name: Some(format!("{}_material_{}_texture", name_prefix, material_id)),
            sampler: Some(self.sampler),
            source: image_index,
            extensions: None,
            extras: Default::default(),
        });

        let material_index = Index::new(root.materials.len() as u32);
        root.materials.push(material::Material {
            name: Some(format!("{}_material_{}", name_prefix, material_id)),
            alpha_cutoff: material
                .alpha_test
                .map(|alpha_ref| material::AlphaCutoff(alpha_ref as f32 / 256.0)),
            alpha_mode: Checked::Valid(if material.alpha_test.is_some() {
                material::AlphaMode::Mask
            } else if material.alpha_enabled {
                material::AlphaMode::Blend
            } else {
                material::AlphaMode::Opaque
            }),
            double_sided: material.two_sided,
            pbr_metallic_roughness: material::PbrMetallicRoughness {
                base_color_factor: material::PbrBaseColorFactor([1.0, 1.0, 1.0, 1.0]),
                base_color_texture: Some(texture::Info {
                    index: texture_index,
                    tex_coord: 0,
                    extensions: None,
                    extras: Default::default(),
                }),
                metallic_factor: material::StrengthFactor(0.0),
                roughness_factor: material::StrengthFactor(1.0),
                metallic_roughness_texture: None,
                extensions: None,
                extras: Default::default(),
            },
            normal_texture: None,
            occlusion_texture: None,
            emissive_texture: None,
            emissive_factor: material::EmissiveFactor([0.0, 0.0, 0.0]),
            extensions: None,
            extras: Default::default(),
        });
        Some(material_index)
    }
}
