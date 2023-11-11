use std::{
    borrow::Cow,
    ffi::OsStr,
    path::{Path, PathBuf},
};

use bytes::{BufMut, BytesMut};
use clap::Parser;
use gltf_json::{buffer, scene, texture, validation::Checked, Index};
use roselib::{
    files::{STB, ZMD, ZMO, ZMS, ZON, ZSC},
    io::RoseFile,
};

mod object_list;
use object_list::ObjectList;

mod mesh_builder;

mod mesh;
use mesh::load_mesh;

mod skeletal_animation;
use serde_json::value::RawValue;
use skeletal_animation::{load_skeletal_animation, load_skeleton};

mod zone;
use zone::load_zone;

/// Converts ROSE files to a .gltf file
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Output file path
    #[arg(short, long = "out")]
    output: PathBuf,

    /// List of input files
    input: Vec<PathBuf>,

    /// When converting a zon, only use blocks with this x value.
    #[arg(long)]
    filter_block_x: Option<i32>,

    /// When converting a zon, only use blocks with this y value.
    #[arg(long)]
    filter_block_y: Option<i32>,

    /// Choose better triangulation for heightmaps, though it may not match your ROSE client.
    #[arg(long, default_value_t = true)]
    use_better_heightmap_triangles: bool,
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
            .map_or(false, |s| OsStr::new("3ddata").eq_ignore_ascii_case(s))
        {
            return parent_path.parent().map(|p| p.to_path_buf());
        }

        path = parent_path;
    }

    None
}

fn main() {
    let args = Args::parse();

    let output_file_path = args.output;
    let input_files = args.input;

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
                let mut deco = ObjectList::new(
                    ZSC::from_path(
                        &assets_path.join(Path::new(list_zone.value(zone_id, 12).unwrap())),
                    )
                    .expect("Failed to read deco zsc"),
                    sampler_index,
                );
                let mut cnst = ObjectList::new(
                    ZSC::from_path(
                        &assets_path.join(Path::new(list_zone.value(zone_id, 13).unwrap())),
                    )
                    .expect("Failed to read cnst zsc"),
                    sampler_index,
                );

                load_zone(
                    &mut root,
                    &mut binary_data,
                    &zon,
                    assets_path,
                    map_path,
                    &mut deco,
                    &mut cnst,
                    args.use_better_heightmap_triangles,
                    args.filter_block_x,
                    args.filter_block_y,
                );
            }
            unknown => {
                panic!("Unsupported file extension {}", unknown);
            }
        }
    }

    pad_align(&mut binary_data);
    root.buffers.push(buffer::Buffer {
        name: None,
        byte_length: binary_data.len() as u32,
        extensions: Default::default(),
        extras: Default::default(),
        uri: None,
    });

    let json_string = gltf_json::serialize::to_string(&root).expect("Serialization error");
    let json_length = (json_string.len() as u32 + 3) & !3;
    let glb = gltf::binary::Glb {
        header: gltf::binary::Header {
            magic: *b"glTF",
            version: 2,
            length: json_length + binary_data.len() as u32,
        },
        bin: Some(Cow::Borrowed(binary_data.as_ref())),
        json: Cow::Owned(json_string.into_bytes()),
    };
    let writer = std::fs::File::create(output_file_path).expect("I/O error");
    glb.to_writer(writer).expect("glTF binary output error");
}
