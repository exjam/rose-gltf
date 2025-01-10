use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    ffi::OsStr,
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::Context;
use bytes::{BufMut, BytesMut};
use glam::{Quat, Vec3};
use gltf::{
    animation::{
        util::{ReadOutputs, Rotations},
        Interpolation,
    },
    mesh::util::{ReadColors, ReadIndices, ReadJoints, ReadTexCoords, ReadWeights},
};
use gltf_json::{
    buffer, scene, texture,
    validation::{Checked, USize64},
    Index,
};
use rose_file_lib::{
    files::{
        zmd::Bone,
        zms::{Vertex, VertexFormat},
        STB, ZMD, ZMO, ZMS, ZON, ZSC,
    },
    io::RoseFile,
    utils::{Quaternion, Vector3},
};

mod object_list;
use object_list::ObjectList;

mod mesh_builder;

mod mesh;
use mesh::load_mesh;

mod animation;
mod skeletal_animation;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use skeletal_animation::{load_skeletal_animation, load_skeleton};

mod zone;
use zone::load_zone;

// Exports
pub use rose_file_lib;

pub struct GltfData {
    pub document: gltf::Document,
    pub buffers: Vec<gltf::buffer::Data>,
    pub images: Vec<gltf::image::Data>,
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct RoseGltfConvOptions {
    /// When converting a zon, only use blocks with this x value.
    pub filter_block_x: Option<i32>,

    /// When converting a zon, only use blocks with this y value.
    pub filter_block_y: Option<i32>,

    /// Choose better triangulation for heightmaps, though it may not match your ROSE client.
    pub use_better_heightmap_triangles: bool,
}

fn pad_align(binary_data: &mut BytesMut) {
    while binary_data.len() % 4 != 0 {
        binary_data.put_u8(0);
    }
}

fn find_assets_root_path(file_path: &Path) -> Option<PathBuf> {
    let mut path = file_path;
    while let Some(parent_path) = path.parent() {
        if parent_path
            .file_name()
            .is_some_and(|s| OsStr::new("3ddata").eq_ignore_ascii_case(s))
        {
            return parent_path.parent().map(|p| p.to_path_buf());
        }

        path = parent_path;
    }

    None
}

pub fn rose_to_gltf(
    input_files: &[PathBuf],
    options: &RoseGltfConvOptions,
) -> anyhow::Result<gltf::Gltf> {
    // Sort the files so we always load skeletons first so we have skeleton first
    let mut input_files = input_files.to_vec();
    input_files.sort_by(|a, b| {
        let ext_a = a.extension().and_then(|ext| ext.to_str()).unwrap_or("");
        let ext_b = b.extension().and_then(|ext| ext.to_str()).unwrap_or("");
        match (ext_a, ext_b) {
            ("zmd", "zmd") => std::cmp::Ordering::Equal,
            ("zmd", _) => std::cmp::Ordering::Less,
            (_, "zmd") => std::cmp::Ordering::Greater,
            ("zmo", "zmo") => std::cmp::Ordering::Equal,
            ("zmo", _) => std::cmp::Ordering::Less,
            (_, "zmo") => std::cmp::Ordering::Greater,
            ("zms", "zms") => std::cmp::Ordering::Equal,
            ("zms", _) => std::cmp::Ordering::Less,
            (_, "zms") => std::cmp::Ordering::Greater,
            (ext_a, ext_b) => ext_a.cmp(ext_b),
        }
    });

    let mut binary_data = BytesMut::with_capacity(8 * 1024 * 1024);
    let mut root = gltf_json::Root::default();
    root.scenes.push(gltf_json::Scene {
        name: None,
        extensions: Default::default(),
        extras: Some(
            RawValue::from_string(
                r#"{
                    "TLM_SceneProperties": {
                        "tlm_encoding_use": 1,
                        "tlm_encoding_mode_a": 2,
                        "tlm_format": 1
                    },
                    "TLM_EngineProperties": {
                      "tlm_mode": 1,
                      "tlm_quality": 4,
                      "tlm_resolution_scale": 0
                    }
                }"#
                .to_string(),
            )
            .unwrap(),
        ),
        nodes: Default::default(),
    });

    let mut skin_index = None;

    for file_path in input_files {
        let file_name = file_path
            .file_stem()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default()
            .to_string();

        let file_extension = file_path
            .extension()
            .unwrap_or_default()
            .to_ascii_lowercase()
            .to_str()
            .unwrap_or_default()
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
                root.nodes.push(scene::Node {
                    name: Some(format!("{}_node", file_name)),
                    camera: None,
                    children: None,
                    extensions: Default::default(),
                    extras: Default::default(),
                    matrix: None,
                    mesh: Some(Index::new(mesh_index)),
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
                root.scenes[0].nodes.push(Index::new(node_index));
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

                let deco_models = ZSC::from_path(
                    &assets_path.join(Path::new(list_zone.value(zone_id, 12).unwrap())),
                )
                .expect("Failed to read deco zsc");
                let cnst_models = ZSC::from_path(
                    &assets_path.join(Path::new(list_zone.value(zone_id, 13).unwrap())),
                )
                .expect("Failed to read cnst zsc");

                // Create a sampler for deco + cnst to use.
                let sampler_index = Index::<texture::Sampler>::new(root.samplers.len() as u32);
                root.samplers.push(texture::Sampler {
                    name: Some("default_sampler".to_string()),
                    mag_filter: Some(Checked::Valid(texture::MagFilter::Linear)),
                    min_filter: Some(Checked::Valid(texture::MinFilter::LinearMipmapLinear)),
                    wrap_s: Checked::Valid(texture::WrappingMode::ClampToEdge),
                    wrap_t: Checked::Valid(texture::WrappingMode::ClampToEdge),
                    extensions: None,
                    extras: Default::default(),
                });

                let zon = ZON::from_path(&file_path).expect("Failed to load ZON");
                let mut deco = ObjectList::new(deco_models, sampler_index);
                let mut cnst = ObjectList::new(cnst_models, sampler_index);

                if let Err(e) = load_zone(
                    &mut root,
                    &mut binary_data,
                    &zon,
                    assets_path,
                    map_path,
                    &mut deco,
                    &mut cnst,
                    options.use_better_heightmap_triangles,
                    options.filter_block_x,
                    options.filter_block_y,
                ) {
                    eprintln!("{:?}", e);
                }
            }
            _ => {
                anyhow::bail!("Unsupported file extension {}", &file_path.display());
            }
        }
    }

    pad_align(&mut binary_data);

    root.buffers.push(buffer::Buffer {
        name: None,
        byte_length: USize64::from(binary_data.len()),
        extensions: Default::default(),
        extras: Default::default(),
        uri: None,
    });

    let gltf = gltf::Gltf {
        document: gltf::Document::from_json(root)?,
        blob: Some(binary_data.to_vec()),
    };

    Ok(gltf)
}

#[derive(Default, Clone, PartialEq, Serialize, Deserialize)]
pub enum GltfFormat {
    #[default]
    Binary,
    Text,
}

impl GltfFormat {
    pub fn file_extension(&self) -> &str {
        match self {
            GltfFormat::Binary => "glb",
            GltfFormat::Text => "gltf",
        }
    }
}

pub fn save_gltf(gltf: &gltf::Gltf, output_path: &Path, format: &GltfFormat) -> anyhow::Result<()> {
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .context("Failed to create intermediate directories for output")?;
    }

    match format {
        // Save as GLTF json + binary
        GltfFormat::Text => {
            let mut root = gltf.document.clone().into_json();

            for buffer in root.buffers.iter_mut() {
                buffer.uri = output_path
                    .file_stem()
                    .map(|s| format!("{}.bin", s.to_string_lossy()));
            }

            let writer = fs::File::create(output_path)
                .context(format!("Failed to create file: {}", &output_path.display()))?;

            gltf_json::serialize::to_writer_pretty(writer, &root).context("Serialization error")?;

            if let Some(blob) = &gltf.blob {
                let mut writer =
                    fs::File::create(output_path.with_extension("bin")).context("I/O error")?;
                writer.write_all(blob).context("I/O error")?;
            }
        }
        GltfFormat::Binary => {
            let json_string = gltf_json::serialize::to_string(gltf.document.as_json())
                .context("Serialization error")?;
            let json_length = (json_string.len() as u32 + 3) & !3;

            let (bin, bin_len) = gltf.blob.as_ref().map_or((None, 0), |blob| {
                (Some(Cow::Borrowed(blob.as_ref())), blob.len())
            });

            let json = Cow::Owned(json_string.into_bytes());

            let glb = gltf::binary::Glb {
                header: gltf::binary::Header {
                    magic: *b"glTF",
                    version: 2,
                    length: json_length + bin_len as u32,
                },
                bin,
                json,
            };

            let writer = std::fs::File::create(output_path).context("I/O error")?;
            glb.to_writer(writer).context("glTF binary output error")?;
        }
    }

    Ok(())
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct GltfRoseConvOptions {
    /// FPS to use for ZMO
    pub zmo_fps: u32,
}

#[derive(Default)]
pub struct GltfRoseResult {
    pub zms: Vec<(String, ZMS)>,
    pub zmd: Vec<(String, ZMD)>,
    pub zmo: Vec<(String, ZMO)>,
}

impl GltfRoseResult {
    pub fn save_to_dir(&mut self, output: &Path) -> anyhow::Result<()> {
        fs::create_dir_all(output).context(format!(
            "Failed to create intermediate dirs: {}",
            output.display()
        ))?;

        let sanitize_name = |name: &str| -> String {
            let invalid_chars: &[char] = &['/', '\\', ':', '*', '?', '"', '<', '>', '|', '\0', '.'];
            name.chars()
                .map(|c| if invalid_chars.contains(&c) { '_' } else { c })
                .collect()
        };

        for (zms_name, zms) in self.zms.iter_mut() {
            let p = output.join(sanitize_name(zms_name)).with_extension("zms");
            let f = fs::File::create(&p)
                .context(format!("Failed to create zms file: {}", p.display()))?;
            zms.write_to_file(&f)
                .context(format!("Failed to write zms file: {}", p.display()))?;
        }

        for (zmo_name, zmo) in self.zmo.iter_mut() {
            let p = output.join(sanitize_name(zmo_name)).with_extension("zmo");
            let f = fs::File::create(&p)
                .context(format!("Failed to create zmo file: {}", p.display()))?;
            zmo.write_to_file(&f)
                .context(format!("Failed to write zmo file: {}", p.display()))?;
        }

        for (zmd_name, zmd) in self.zmd.iter_mut() {
            let p = output.join(sanitize_name(zmd_name)).with_extension("zmd");
            let f = fs::File::create(&p)
                .context(format!("Failed to create zmd file: {}", p.display()))?;
            zmd.write_to_file(&f)
                .context(format!("Failed to write zmd file: {}", p.display()))?;
        }

        Ok(())
    }
}

#[derive(Debug)]
enum ZmdBoneIndex {
    Bone(usize),
    Dummy(usize),
}

pub fn gltf_to_rose(
    gltf_data: &GltfData,
    options: &GltfRoseConvOptions,
) -> anyhow::Result<GltfRoseResult> {
    let mut result = GltfRoseResult::default();

    let animation_fps = options.zmo_fps;

    let mut dummy_nodes = Vec::new();
    let mut processed_meshes = HashSet::new();

    for node in gltf_data.document.nodes() {
        // Skip dummy nodes but save them to be used in ZMD later
        if let Some(name) = node.name() {
            if name.starts_with("dummy_") {
                dummy_nodes.push(node);
                continue;
            }
        }

        // Skip anything that doesn't have a mesh
        let Some(mesh) = node.mesh() else {
            continue;
        };

        // Skip meshes we've already processed
        if !processed_meshes.insert(mesh.index()) {
            continue;
        }

        let primitive = mesh.primitives().next().context(format!(
            "Expected mesh to have 1 primitive. Index: {}, name: {}",
            mesh.index(),
            mesh.name().unwrap_or("None")
        ))?;

        let mut zms = ZMS::new();
        let reader = primitive.reader(|buffer| Some(&gltf_data.buffers[buffer.index()]));

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

            // Skeleton can contain more than 48 bones but mesh should not
            // exceed this number so we narrow down the bone list to only what
            // the mesh actually uses.
            let mut bones_used = HashSet::new();
            for vertex in zms.vertices.iter_mut() {
                bones_used.insert(vertex.bone_indices.x);
                bones_used.insert(vertex.bone_indices.y);
                bones_used.insert(vertex.bone_indices.z);
                bones_used.insert(vertex.bone_indices.w);
            }

            if bones_used.len() > 48 {
                anyhow::bail!("A mesh can only bind to a maximum of 48 bones");
            }

            if node.skin().is_none() {
                anyhow::bail!("Mesh has bone weights but is not assocated with a skin");
            };

            // Map from the bone index in the skeleton to the index of the bone
            // indices list in the mesh
            let mut bone_map = HashMap::new();
            for bone_idx in bones_used {
                bone_map.insert(bone_idx, zms.bones.len());
                zms.bones.push(bone_idx);
            }

            for vertex in zms.vertices.iter_mut() {
                if let Some(new_idx) = bone_map.get(&vertex.bone_indices.x) {
                    vertex.bone_indices.x = *new_idx as i16;
                } else {
                    vertex.bone_indices.x = 0;
                    vertex.bone_weights.x = 0.0;
                }

                if let Some(new_idx) = bone_map.get(&vertex.bone_indices.y) {
                    vertex.bone_indices.y = *new_idx as i16;
                } else {
                    vertex.bone_indices.y = 0;
                    vertex.bone_weights.y = 0.0;
                }

                if let Some(new_idx) = bone_map.get(&vertex.bone_indices.z) {
                    vertex.bone_indices.z = *new_idx as i16;
                } else {
                    vertex.bone_indices.z = 0;
                    vertex.bone_weights.z = 0.0;
                }

                if let Some(new_idx) = bone_map.get(&vertex.bone_indices.w) {
                    vertex.bone_indices.w = *new_idx as i16;
                } else {
                    vertex.bone_indices.w = 0;
                    vertex.bone_weights.w = 0.0;
                }
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

        result.zms.push((
            mesh.name()
                .map(|s| s.to_string())
                .unwrap_or(format!("mesh_{}", mesh.index())),
            zms,
        ));
    }

    for (animation_index, animation) in gltf_data.document.animations().enumerate() {
        let mut zmo = ZMO::new();
        let mut max_keyframe_time = 0.0f32;

        for channel in animation.channels() {
            let reader = channel.reader(|buffer| Some(&gltf_data.buffers[buffer.index()]));
            for t in reader.read_inputs().unwrap() {
                max_keyframe_time = max_keyframe_time.max(t);
            }
        }

        let num_frames = (max_keyframe_time * animation_fps as f32).ceil() as u32;
        zmo.identifier = "ZMO0002".into();
        zmo.fps = animation_fps;
        zmo.frames = num_frames;

        for channel in animation.channels() {
            let reader = channel.reader(|buffer| Some(&gltf_data.buffers[buffer.index()]));
            let outputs = reader.read_outputs().unwrap();
            let inputs = reader.read_inputs().unwrap();
            let interpolation = channel.sampler().interpolation();
            let target_node = channel.target().node();

            let target_bone_index = gltf_data
                .document
                .skins()
                .flat_map(|skin| skin.joints().enumerate())
                .find_map(|(joint_index, joint_node)| {
                    (target_node.index() == joint_node.index()).then_some(joint_index as u32)
                });

            let Some(target_bone_index) = target_bone_index else {
                continue;
            };

            match outputs {
                ReadOutputs::Translations(translations) => {
                    let keyframes: Vec<_> =
                        inputs.zip(translations.map(glam::Vec3::from)).collect();
                    let mut rasterized_frames = Vec::with_capacity(num_frames as usize);

                    for frame_index in 0..num_frames {
                        let frame_time = frame_index as f32 / animation_fps as f32;

                        let keyframe_before = keyframes
                            .iter()
                            .rfind(|(t, _)| *t <= frame_time)
                            .unwrap_or_else(|| keyframes.first().unwrap());
                        let keyframe_after = keyframes
                            .iter()
                            .find(|(t, _)| *t >= frame_time)
                            .unwrap_or_else(|| keyframes.last().unwrap());

                        let value = match interpolation {
                            Interpolation::Linear => {
                                if keyframe_before == keyframe_after {
                                    keyframe_before.1
                                } else {
                                    let lerp_factor = (frame_time - keyframe_before.0)
                                        / (keyframe_after.0 - keyframe_before.0);
                                    keyframe_before.1.lerp(keyframe_after.1, lerp_factor)
                                }
                            }
                            Interpolation::Step => keyframe_before.1,
                            Interpolation::CubicSpline => todo!(),
                        } * 100.0;

                        rasterized_frames.push(Vector3 {
                            x: value.x,
                            y: -value.z,
                            z: value.y,
                        });
                    }

                    zmo.channels.push(rose_file_lib::files::zmo::Channel {
                        typ: rose_file_lib::files::zmo::ChannelType::Position,
                        index: target_bone_index,
                        frames: rose_file_lib::files::zmo::ChannelData::Position(rasterized_frames),
                    });
                }
                ReadOutputs::Rotations(rotations) => {
                    let rotations: Vec<glam::Quat> = match rotations {
                        Rotations::I8(normalized) => normalized
                            .map(|xyzw| xyzw.map(|n| n as f32 / 127.0))
                            .map(glam::Quat::from_array)
                            .collect(),
                        Rotations::U8(normalized) => normalized
                            .map(|xyzw| xyzw.map(|n| n as f32 / 255.0))
                            .map(glam::Quat::from_array)
                            .collect(),
                        Rotations::I16(normalized) => normalized
                            .map(|xyzw| xyzw.map(|n| n as f32 / 32767.0))
                            .map(glam::Quat::from_array)
                            .collect(),
                        Rotations::U16(normalized) => normalized
                            .map(|xyze| xyze.map(|n| n as f32 / 65535.0))
                            .map(glam::Quat::from_array)
                            .collect(),
                        Rotations::F32(xyzw) => xyzw.map(glam::Quat::from_array).collect(),
                    };

                    let keyframes: Vec<_> = inputs.zip(rotations).collect();
                    let mut rasterized_frames = Vec::with_capacity(num_frames as usize);

                    for frame_index in 0..num_frames {
                        let frame_time = frame_index as f32 / animation_fps as f32;

                        let keyframe_before = keyframes
                            .iter()
                            .rfind(|(t, _)| *t <= frame_time)
                            .unwrap_or_else(|| keyframes.first().unwrap());
                        let keyframe_after = keyframes
                            .iter()
                            .find(|(t, _)| *t >= frame_time)
                            .unwrap_or_else(|| keyframes.last().unwrap());

                        let value = match interpolation {
                            Interpolation::Linear => {
                                if keyframe_before == keyframe_after {
                                    keyframe_before.1
                                } else {
                                    let lerp_factor = (frame_time - keyframe_before.0)
                                        / (keyframe_after.0 - keyframe_before.0);
                                    keyframe_before.1.slerp(keyframe_after.1, lerp_factor)
                                }
                            }
                            Interpolation::Step => keyframe_before.1,
                            Interpolation::CubicSpline => todo!(),
                        };
                        let value =
                            glam::Quat::from_xyzw(value.x, -value.z, value.y, value.w).normalize();

                        rasterized_frames.push(rose_file_lib::utils::Quaternion {
                            x: value.x,
                            y: value.y,
                            z: value.z,
                            w: value.w,
                        });
                    }

                    zmo.channels.push(rose_file_lib::files::zmo::Channel {
                        typ: rose_file_lib::files::zmo::ChannelType::Rotation,
                        index: target_bone_index,
                        frames: rose_file_lib::files::zmo::ChannelData::Rotation(rasterized_frames),
                    });
                }
                ReadOutputs::Scales(scales) => {
                    let keyframes: Vec<_> = inputs.zip(scales.map(glam::Vec3::from)).collect();
                    let mut rasterized_frames = Vec::with_capacity(num_frames as usize);

                    for frame_index in 0..num_frames {
                        let frame_time = frame_index as f32 / animation_fps as f32;

                        let keyframe_before = keyframes
                            .iter()
                            .rfind(|(t, _)| *t <= frame_time)
                            .unwrap_or_else(|| keyframes.first().unwrap());
                        let keyframe_after = keyframes
                            .iter()
                            .find(|(t, _)| *t >= frame_time)
                            .unwrap_or_else(|| keyframes.last().unwrap());

                        let value = match interpolation {
                            Interpolation::Linear => {
                                let lerp_factor = (frame_time - keyframe_before.0)
                                    / (keyframe_after.0 - keyframe_before.0);
                                keyframe_before.1.lerp(keyframe_after.1, lerp_factor)
                            }
                            Interpolation::Step => keyframe_before.1,
                            Interpolation::CubicSpline => todo!(),
                        };

                        rasterized_frames.push((value.x + value.y + value.z) / 3.0);
                    }

                    zmo.channels.push(rose_file_lib::files::zmo::Channel {
                        typ: rose_file_lib::files::zmo::ChannelType::Scale,
                        index: target_bone_index,
                        frames: rose_file_lib::files::zmo::ChannelData::Scale(rasterized_frames),
                    });
                }
                _ => {}
            }
        }

        result.zmo.push((
            animation
                .name()
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("animation_{}", animation_index)),
            zmo,
        ));
    }

    let node_to_bone = |node: &gltf::Node, name: &str| -> Bone {
        let (translation, rotation, _scale) = node.transform().decomposed();

        let translation = Vec3::from_array(translation) * 100.0;
        let rotation = Quat::from_array(rotation);

        Bone {
            parent: 0,
            name: name.to_string(),
            position: Vector3 {
                x: translation.x,
                y: -translation.z,
                z: translation.y,
            },
            rotation: Quaternion {
                x: rotation.x,
                y: -rotation.z,
                z: rotation.y,
                w: rotation.w,
            },
        }
    };

    for (skin_index, skin) in gltf_data.document.skins().enumerate() {
        let mut zmd = ZMD::new();

        let joints: Vec<gltf::Node> = skin.joints().collect();
        let mut node_to_zmd_idx = HashMap::new();

        for joint in joints.iter() {
            let bone_name = joint
                .name()
                .map(|s| s.to_string())
                .unwrap_or(format!("bone_{}", zmd.bones.len()));

            let bone = node_to_bone(joint, &bone_name);

            node_to_zmd_idx.insert(joint.index(), ZmdBoneIndex::Bone(zmd.bones.len()));
            zmd.bones.push(bone);
        }

        for dummy_node in &dummy_nodes {
            let bone_name = dummy_node
                .name()
                .map(|s| s.to_string())
                .unwrap_or(format!("dummy_{}", zmd.dummy_bones.len()));

            let bone = node_to_bone(dummy_node, &bone_name);

            node_to_zmd_idx.insert(
                dummy_node.index(),
                ZmdBoneIndex::Dummy(zmd.dummy_bones.len()),
            );
            zmd.dummy_bones.push(bone);
        }

        for parent in joints.iter().chain(dummy_nodes.iter()) {
            for child in parent.children() {
                let parent_idx = match node_to_zmd_idx.get(&parent.index()) {
                    Some(ZmdBoneIndex::Bone(idx)) => *idx,
                    _ => anyhow::bail!("Dummy bones should not have children"),
                };

                let Some(child_index) = node_to_zmd_idx.get(&child.index()) else {
                    continue;
                };

                match child_index {
                    ZmdBoneIndex::Bone(child_idx) => {
                        zmd.bones[*child_idx].parent = parent_idx as i32;
                    }
                    ZmdBoneIndex::Dummy(child_idx) => {
                        zmd.dummy_bones[*child_idx].parent = parent_idx as i32;
                    }
                }
            }
        }

        // Best effort sort dummy bones by name
        zmd.dummy_bones.sort_by(|a, b| {
            let a_num = a
                .name
                .split('_')
                .nth(1)
                .unwrap_or_default()
                .parse::<u32>()
                .unwrap_or(0);
            let b_num = b
                .name
                .split('_')
                .nth(1)
                .unwrap_or_default()
                .parse::<u32>()
                .unwrap_or(0);
            a_num.cmp(&b_num)
        });

        result.zmd.push((
            skin.name()
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("skeleton_{}", skin_index)),
            zmd,
        ));
    }

    Ok(result)
}
