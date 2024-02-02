use std::{
    io::Cursor,
    path::{Path, PathBuf},
};

use bytes::{BufMut, BytesMut};
use glam::{EulerRot, Quat, Vec2, Vec3};
use gltf_json::{
    buffer, extensions, material, mesh,
    scene::{self, UnitQuaternion},
    texture,
    validation::{Checked, USize64},
    Index,
};
use roselib::{
    files::{him::Heightmap, ifo::MapData, til::Tilemap, zon, HIM, IFO, TIL, ZMO},
    io::RoseFile,
};
use serde_json::value::RawValue;

use crate::{
    mesh_builder::{MeshBuilder, MeshData},
    object_list::ObjectList,
    pad_align,
};

struct BlockData {
    pub block_x: i32,
    pub block_y: i32,
    pub ifo: MapData,
    pub him: Heightmap,
    pub til: Tilemap,
}

fn convert_position(position: roselib::utils::Vector3<f32>) -> [f32; 3] {
    [position.x / 100.0, position.z / 100.0, -position.y / 100.0]
}

fn convert_scale(scale: roselib::utils::Vector3<f32>) -> [f32; 3] {
    [scale.x, scale.z, scale.y]
}

fn convert_rotation(rotation: roselib::utils::Quaternion) -> UnitQuaternion {
    UnitQuaternion([rotation.x, rotation.z, -rotation.y, rotation.w])
}

fn generate_terrain_materials(
    root: &mut gltf_json::Root,
    binary_data: &mut BytesMut,
    zon: &zon::Zone,
    assets_path: &Path,
    blocks: &[BlockData],
) -> Vec<Index<material::Material>> {
    let texture_size = 1024;
    let texture_tile_size = texture_size / 16;
    let mut tile_images = Vec::with_capacity(zon.textures.len());

    for tile_texure_path in zon.textures.iter() {
        if tile_texure_path == "end" {
            break;
        }

        let mut tile_image =
            image::open(assets_path.join(tile_texure_path)).expect("Failed to load DDS");
        if tile_image.width() != texture_tile_size {
            tile_image = tile_image.resize(
                texture_tile_size,
                texture_tile_size,
                image::imageops::FilterType::Triangle,
            );
        }
        tile_images.push(tile_image.to_rgba8());
    }

    let sampler_index = Index::<texture::Sampler>::new(root.samplers.len() as u32);
    root.samplers.push(texture::Sampler {
        name: Some("terrain_sampler".to_string()),
        mag_filter: Some(Checked::Valid(texture::MagFilter::Linear)),
        min_filter: Some(Checked::Valid(texture::MinFilter::LinearMipmapLinear)),
        wrap_s: Checked::Valid(texture::WrappingMode::ClampToEdge),
        wrap_t: Checked::Valid(texture::WrappingMode::ClampToEdge),
        extensions: None,
        extras: Default::default(),
    });

    let mut block_materials = Vec::new();
    for block in blocks.iter() {
        let mut image = image::RgbImage::new(texture_size, texture_size);

        // Rasterise the tilemap to a single image
        for tile_x in 0..16 {
            for tile_y in 0..16 {
                let tile = &zon.tiles[block.til.tiles[tile_y][tile_x].tile_id as usize];
                let tile_index1 = (tile.layer1 + tile.offset1) as usize;
                let tile_index2 = (tile.layer2 + tile.offset2) as usize;
                let tile_image1 = tile_images.get(tile_index1).unwrap();
                let tile_image2 = tile_images.get(tile_index2).unwrap();

                fn lerp(a: u8, b: u8, x: u8) -> u8 {
                    ((a as u16 * (256 - x as u16) + b as u16 * x as u16) >> 8) as u8
                }

                let dst_x = tile_x as u32 * texture_tile_size;
                let dst_y = tile_y as u32 * texture_tile_size;
                match tile.rotation {
                    zon::ZoneTileRotation::Unknown | zon::ZoneTileRotation::None => {
                        for y in 0..texture_tile_size {
                            for x in 0..texture_tile_size {
                                let pixel1 = tile_image1.get_pixel(x, y);
                                let pixel2 = tile_image2.get_pixel(x, y);
                                image.put_pixel(
                                    dst_x + x,
                                    dst_y + y,
                                    image::Rgb([
                                        lerp(pixel1[0], pixel2[0], pixel2[3]),
                                        lerp(pixel1[1], pixel2[1], pixel2[3]),
                                        lerp(pixel1[2], pixel2[2], pixel2[3]),
                                    ]),
                                );
                            }
                        }
                    }
                    zon::ZoneTileRotation::FlipHorizontal => {
                        for y in 0..texture_tile_size {
                            for x in 0..texture_tile_size {
                                let pixel1 = tile_image1.get_pixel(x, y);
                                let pixel2 = tile_image2.get_pixel(texture_tile_size - 1 - x, y);
                                image.put_pixel(
                                    dst_x + x,
                                    dst_y + y,
                                    image::Rgb([
                                        lerp(pixel1[0], pixel2[0], pixel2[3]),
                                        lerp(pixel1[1], pixel2[1], pixel2[3]),
                                        lerp(pixel1[2], pixel2[2], pixel2[3]),
                                    ]),
                                );
                            }
                        }
                    }
                    zon::ZoneTileRotation::FlipVertical => {
                        for y in 0..texture_tile_size {
                            for x in 0..texture_tile_size {
                                let pixel1 = tile_image1.get_pixel(x, y);
                                let pixel2 = tile_image2.get_pixel(x, texture_tile_size - 1 - y);
                                image.put_pixel(
                                    dst_x + x,
                                    dst_y + y,
                                    image::Rgb([
                                        lerp(pixel1[0], pixel2[0], pixel2[3]),
                                        lerp(pixel1[1], pixel2[1], pixel2[3]),
                                        lerp(pixel1[2], pixel2[2], pixel2[3]),
                                    ]),
                                );
                            }
                        }
                    }
                    zon::ZoneTileRotation::Flip => {
                        for y in 0..texture_tile_size {
                            for x in 0..texture_tile_size {
                                let pixel1 = tile_image1.get_pixel(x, y);
                                let pixel2 = tile_image2.get_pixel(
                                    texture_tile_size - 1 - x,
                                    texture_tile_size - 1 - y,
                                );
                                image.put_pixel(
                                    dst_x + x,
                                    dst_y + y,
                                    image::Rgb([
                                        lerp(pixel1[0], pixel2[0], pixel2[3]),
                                        lerp(pixel1[1], pixel2[1], pixel2[3]),
                                        lerp(pixel1[2], pixel2[2], pixel2[3]),
                                    ]),
                                );
                            }
                        }
                    }
                    zon::ZoneTileRotation::Clockwise90 => {
                        for y in 0..texture_tile_size {
                            for x in 0..texture_tile_size {
                                let pixel1 = tile_image1.get_pixel(x, y);
                                let pixel2 = tile_image2.get_pixel(y, texture_tile_size - 1 - x);
                                image.put_pixel(
                                    dst_x + x,
                                    dst_y + y,
                                    image::Rgb([
                                        lerp(pixel1[0], pixel2[0], pixel2[3]),
                                        lerp(pixel1[1], pixel2[1], pixel2[3]),
                                        lerp(pixel1[2], pixel2[2], pixel2[3]),
                                    ]),
                                );
                            }
                        }
                    }
                    zon::ZoneTileRotation::CounterClockwise90 => {
                        for y in 0..texture_tile_size {
                            for x in 0..texture_tile_size {
                                let pixel1 = tile_image1.get_pixel(x, y);
                                let pixel2 = tile_image2.get_pixel(y, x);
                                image.put_pixel(
                                    dst_x + x,
                                    dst_y + y,
                                    image::Rgb([
                                        lerp(pixel1[0], pixel2[0], pixel2[3]),
                                        lerp(pixel1[1], pixel2[1], pixel2[3]),
                                        lerp(pixel1[2], pixel2[2], pixel2[3]),
                                    ]),
                                );
                            }
                        }
                    }
                }
            }
        }

        let (texture_data_start, texture_data_length) = {
            let mut buffer: Vec<u8> = Vec::new();
            image
                .write_to(&mut Cursor::new(&mut buffer), image::ImageOutputFormat::Png)
                .expect("Failed to write PNG");
            pad_align(binary_data);
            let texture_data_start = binary_data.len() as u32;
            binary_data.put_slice(&buffer);
            pad_align(binary_data);
            (
                texture_data_start,
                binary_data.len() as u32 - texture_data_start,
            )
        };

        let buffer_index = Index::new(root.buffer_views.len() as u32);
        root.buffer_views.push(buffer::View {
            name: Some(format!(
                "{}_{}_tilemap_image_buffer",
                block.block_x, block.block_y,
            )),
            buffer: Index::new(0),
            byte_length: USize64::from(texture_data_length as usize),
            byte_offset: Some(USize64::from(texture_data_start as usize)),
            byte_stride: None,
            extensions: Default::default(),
            extras: Default::default(),
            target: None,
        });

        let image_index = Index::new(root.images.len() as u32);
        root.images.push(gltf_json::image::Image {
            name: Some(format!("{}_{}_tilemap_image", block.block_x, block.block_y,)),
            buffer_view: Some(buffer_index),
            mime_type: Some(gltf_json::image::MimeType("image/png".into())),
            uri: None,
            extensions: None,
            extras: Default::default(),
        });

        let texture_index = Index::new(root.textures.len() as u32);
        root.textures.push(texture::Texture {
            name: Some(format!(
                "{}_{}_tilemap_texture",
                block.block_x, block.block_y,
            )),
            sampler: Some(sampler_index),
            source: image_index,
            extensions: None,
            extras: Default::default(),
        });

        let material_index = Index::<material::Material>::new(root.materials.len() as u32);
        root.materials.push(material::Material {
            name: Some(format!(
                "{}_{}_tilemap_material",
                block.block_x, block.block_y,
            )),
            alpha_cutoff: None,
            alpha_mode: Checked::Valid(material::AlphaMode::Opaque),
            double_sided: false,
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

        block_materials.push(material_index);
    }

    block_materials
}

fn generate_terrain_mesh(
    root: &mut gltf_json::Root,
    binary_data: &mut BytesMut,
    block: &BlockData,
    use_better_heightmap_triangles: bool,
) -> MeshData {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    for tile_x in 0..16 {
        for tile_y in 0..16 {
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
                    let heightmap_x = x + tile_x * 4;
                    let heightmap_y = y + tile_y * 4;
                    let height = get_height(&block.him, heightmap_x, heightmap_y);
                    let height_l = get_height(&block.him, heightmap_x - 1, heightmap_y);
                    let height_r = get_height(&block.him, heightmap_x + 1, heightmap_y);
                    let height_t = get_height(&block.him, heightmap_x, heightmap_y - 1);
                    let height_b = get_height(&block.him, heightmap_x, heightmap_y + 1);
                    let normal = Vec3::new(
                        (height_l - height_r) / 2.0,
                        1.0,
                        (height_t - height_b) / 2.0,
                    );

                    positions.push(Vec3::new(
                        tile_offset_x + x as f32 * 2.5,
                        height,
                        tile_offset_y + y as f32 * 2.5,
                    ));
                    normals.push(Vec3::new(normal.x, normal.y, normal.z));
                    uvs.push(Vec2::new(
                        (tile_x as f32 * 4.0 + x as f32) / 64.0,
                        (tile_y as f32 * 4.0 + y as f32) / 64.0,
                    ));
                }
            }

            for y in 0..(5 - 1) {
                for x in 0..(5 - 1) {
                    let start = tile_indices_base + y * 5 + x;
                    let tl = start;
                    let tr = start + 1;
                    let bl = start + 5;
                    let br = start + 1 + 5;

                    // Choose the triangle edge which is shortest
                    let edge_tl_br = (positions[tl as usize].y - positions[br as usize].y).abs();
                    let edge_bl_tr = (positions[bl as usize].y - positions[tr as usize].y).abs();
                    if use_better_heightmap_triangles && edge_tl_br < edge_bl_tr {
                        /*
                         * tl-tr
                         * | \ |
                         * bl-br
                         */
                        indices.push(tl);
                        indices.push(bl);
                        indices.push(br);

                        indices.push(tl);
                        indices.push(br);
                        indices.push(tr);
                    } else {
                        /*
                         * tl-tr
                         * | / |
                         * bl-br
                         */
                        indices.push(tl);
                        indices.push(bl);
                        indices.push(tr);

                        indices.push(tr);
                        indices.push(bl);
                        indices.push(br);
                    }
                }
            }
        }
    }

    let mut mesh_builder = MeshBuilder::new();
    mesh_builder.add_positions(positions);
    mesh_builder.add_normals(normals);
    mesh_builder.add_uv0(uvs.clone());
    mesh_builder.add_uv1(uvs.clone());
    mesh_builder.add_indices(indices);
    mesh_builder.build(
        root,
        binary_data,
        &format!("{}_{}_heightmesh", block.block_x, block.block_y),
    )
}

#[allow(clippy::too_many_arguments)]
pub fn load_zone(
    root: &mut gltf_json::Root,
    binary_data: &mut BytesMut,
    zon: &zon::Zone,
    assets_path: PathBuf,
    map_path: PathBuf,
    deco: &mut ObjectList,
    cnst: &mut ObjectList,
    use_better_heightmap_triangles: bool,
    filter_block_x: Option<i32>,
    filter_block_y: Option<i32>,
) {
    // Add a directional light to the scene
    root.extensions_used.push("KHR_lights_punctual".to_string());
    root.extensions = Some(extensions::Root {
        khr_lights_punctual: Some(extensions::root::KhrLightsPunctual {
            lights: vec![extensions::scene::khr_lights_punctual::Light {
                name: Some("the_sun".to_string()),
                color: [0.88, 0.87, 0.84],
                intensity: 4098.0,
                type_: Checked::Valid(extensions::scene::khr_lights_punctual::Type::Directional),
                range: None,
                spot: None,
                extensions: Default::default(),
                extras: Default::default(),
            }],
        }),
    });
    let light_direction = Quat::from_euler(
        EulerRot::ZYX,
        0.0,
        std::f32::consts::PI * (2.0 / 3.0),
        -std::f32::consts::PI / 4.0,
    );
    let light_node = Index::new(root.nodes.len() as u32);
    root.nodes.push(scene::Node {
        extensions: Some(extensions::scene::Node {
            khr_lights_punctual: Some(extensions::scene::khr_lights_punctual::KhrLightsPunctual {
                light: Index::new(0),
            }),
        }),
        camera: None,
        children: None,
        extras: Default::default(),
        matrix: None,
        mesh: None,
        name: None,
        rotation: Some(UnitQuaternion(light_direction.to_array())),
        scale: Some([1.0, 1.0, 1.0]),
        translation: Some([0.0, 0.0, 0.0]),
        skin: None,
        weights: None,
    });
    root.scenes[0].nodes.push(light_node);

    // Find all blocks
    let mut blocks = Vec::new();
    for block_y in 0..64 {
        for block_x in 0..64 {
            if filter_block_x.is_some() && Some(block_x) != filter_block_x {
                continue;
            }

            if filter_block_y.is_some() && Some(block_y) != filter_block_y {
                continue;
            }

            let ifo = IFO::from_path(&map_path.join(format!("{}_{}.ifo", block_x, block_y)));
            let him = HIM::from_path(&map_path.join(format!("{}_{}.him", block_x, block_y)));
            let til = TIL::from_path(&map_path.join(format!("{}_{}.til", block_x, block_y)));
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

    let mut ocean_material = None;

    // Load all meshes and materials from used objects
    for block in blocks.iter() {
        if !block.ifo.oceans.is_empty() && ocean_material.is_none() {
            ocean_material = Some(Index::new(root.materials.len() as u32));
            root.materials.push(material::Material {
                name: Some("ocean_material".to_string()),
                alpha_cutoff: None,
                alpha_mode: Checked::Valid(material::AlphaMode::Blend),
                double_sided: true,
                pbr_metallic_roughness: material::PbrMetallicRoughness {
                    base_color_factor: material::PbrBaseColorFactor([0.32, 0.46, 0.7, 0.6]),
                    base_color_texture: None,
                    metallic_factor: material::StrengthFactor(0.5),
                    roughness_factor: material::StrengthFactor(0.5),
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
        }

        for block_objects in block.ifo.objects.iter() {
            deco.load_object(
                "deco",
                block_objects.object_id as usize,
                root,
                binary_data,
                &assets_path,
            );
        }

        for block_objects in block.ifo.buildings.iter() {
            cnst.load_object(
                "cnst",
                block_objects.object_id as usize,
                root,
                binary_data,
                &assets_path,
            );
        }
    }

    let block_terrain_materials =
        generate_terrain_materials(root, binary_data, zon, &assets_path, &blocks);

    // Spawn all block nodes
    for (block, block_terrain_material) in blocks.iter().zip(block_terrain_materials.iter()) {
        // Load heightmap
        load_heightmap(
            root,
            binary_data,
            block,
            use_better_heightmap_triangles,
            block_terrain_material,
        );

        // Load ocean patch
        for (ocean_index, ocean) in block.ifo.oceans.iter().enumerate() {
            for (patch_index, patch) in ocean.patches.iter().enumerate() {
                load_ocean_patch(
                    root,
                    binary_data,
                    block,
                    ocean_index,
                    patch_index,
                    patch,
                    ocean_material,
                );
            }
        }

        // Load all deco objects
        for (object_instance_index, object_instance) in block.ifo.objects.iter().enumerate() {
            load_object_instance(
                root,
                binary_data,
                &assets_path,
                block,
                deco,
                "deco",
                object_instance_index,
                object_instance,
            );
        }

        // Load all cnst objects
        for (object_instance_index, object_instance) in block.ifo.buildings.iter().enumerate() {
            load_object_instance(
                root,
                binary_data,
                &assets_path,
                block,
                cnst,
                "cnst",
                object_instance_index,
                object_instance,
            );
        }
    }
}

fn load_ocean_patch(
    root: &mut gltf_json::Root,
    binary_data: &mut BytesMut,
    block: &BlockData,
    ocean_index: usize,
    patch_index: usize,
    patch: &roselib::files::ifo::OceanPatch,
    ocean_material: Option<Index<gltf_json::Material>>,
) {
    let start = Vec3::new(patch.start.x, patch.start.y, -patch.start.z) / 100.0;
    let end = (Vec3::new(patch.end.x, patch.end.y, -patch.end.z) / 100.0) - start;
    let up = Vec3::new(0.0, 1.0, 0.0);

    let mut mesh_builder = MeshBuilder::new();
    mesh_builder.add_positions(vec![
        Vec3::new(0.0, 0.0, end.z),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(end.x, 0.0, 0.0),
        Vec3::new(end.x, 0.0, end.z),
    ]);
    mesh_builder.add_normals(vec![up, up, up, up]);
    mesh_builder.add_indices(vec![0, 2, 1, 0, 3, 2]);
    let mesh_data = mesh_builder.build(
        root,
        binary_data,
        &format!(
            "{}_{}_ocean_{}_{}_mesh",
            block.block_x, block.block_y, ocean_index, patch_index
        ),
    );

    let mesh_index = Index::new(root.meshes.len() as u32);
    root.meshes.push(mesh::Mesh {
        name: Some(format!(
            "{}_{}_ocean_{}_{}_mesh",
            block.block_x, block.block_y, ocean_index, patch_index
        )),
        extensions: Default::default(),
        extras: Default::default(),
        primitives: vec![mesh::Primitive {
            attributes: mesh_data.attributes.clone(),
            extensions: Default::default(),
            extras: Default::default(),
            indices: Some(mesh_data.indices),
            material: ocean_material,
            mode: Checked::Valid(mesh::Mode::Triangles),
            targets: None,
        }],
        weights: None,
    });

    // Spawn a node for a object
    let node_index = Index::new(root.nodes.len() as u32);
    root.nodes.push(scene::Node {
        camera: None,
        children: None,
        extensions: Default::default(),
        extras: Default::default(),
        matrix: None,
        mesh: Some(mesh_index),
        name: Some(format!(
            "{}_{}_ocean_{}_{}",
            block.block_x, block.block_y, ocean_index, patch_index
        )),
        rotation: None,
        scale: Some([1.0, 1.0, 1.0]),
        translation: Some([start.x, start.y, start.z]),
        skin: None,
        weights: None,
    });
    root.scenes[0].nodes.push(node_index);
}

fn load_heightmap(
    root: &mut gltf_json::Root,
    binary_data: &mut BytesMut,
    block: &BlockData,
    use_better_heightmap_triangles: bool,
    block_terrain_material: &Index<gltf_json::Material>,
) {
    let mesh_data = generate_terrain_mesh(root, binary_data, block, use_better_heightmap_triangles);

    let heightmap_mesh = Index::new(root.meshes.len() as u32);
    root.meshes.push(mesh::Mesh {
        name: Some(format!(
            "{}_{}_heightmap_mesh",
            block.block_x, block.block_y
        )),
        extensions: Default::default(),
        extras: Default::default(),
        primitives: vec![mesh::Primitive {
            attributes: mesh_data.attributes,
            extensions: Default::default(),
            extras: Default::default(),
            indices: Some(mesh_data.indices),
            material: Some(*block_terrain_material),
            mode: Checked::Valid(mesh::Mode::Triangles),
            targets: None,
        }],
        weights: None,
    });

    let offset_x = (160.0 * block.block_x as f32) - 5200.0;
    let offset_y = (160.0 * (65.0 - block.block_y as f32)) - 5200.0;
    let node_index = Index::new(root.nodes.len() as u32);
    root.nodes.push(scene::Node {
        camera: None,
        children: None,
        extensions: Default::default(),
        extras: Some(
            RawValue::from_string(format!(
                r#"{{
                "TLM_ObjectProperties": {{
                    "tlm_mesh_lightmap_use": 1,
                    "tlm_mesh_lightmap_resolution": {},
                    "tlm_use_default_channel": 0,
                    "tlm_uv_channel": "UVMap.001"
                }}
            }}"#,
                4
            ))
            .unwrap(),
        ),
        matrix: None,
        mesh: Some(heightmap_mesh),
        name: Some(format!("{}_{}_heightmap", block.block_x, block.block_y,)),
        rotation: Some(UnitQuaternion::default()),
        scale: Some([1.0, 1.0, 1.0]),
        translation: Some([offset_x, 0.0, -offset_y]),
        skin: None,
        weights: None,
    });
    root.scenes[0].nodes.push(node_index);
}

#[allow(clippy::too_many_arguments)]
fn load_object_instance(
    root: &mut gltf_json::Root,
    binary_data: &mut BytesMut,
    assets_path: &Path,
    block: &BlockData,
    object_list: &ObjectList,
    object_list_name: &str,
    object_instance_index: usize,
    object_instance: &roselib::files::ifo::ObjectData,
) {
    let mut children = Vec::new();
    let object_id = object_instance.object_id as usize;
    let object = &object_list.zsc.objects[object_id];
    let object_average_scale =
        (object_instance.scale.x + object_instance.scale.y + object_instance.scale.z) / 3.0;

    // Spawn a node for each object part
    for (part_index, part) in object.parts.iter().enumerate() {
        let mesh_data = object_list.meshes.get(&part.mesh_id).expect("Missing mesh");
        let mesh_index = root.meshes.len() as u32;
        root.meshes.push(mesh::Mesh {
            name: Some(format!(
                "{}_{}_{}_{}_{}_mesh",
                block.block_x, block.block_y, object_list_name, object_instance_index, part_index
            )),
            extensions: Default::default(),
            extras: Default::default(),
            primitives: vec![mesh::Primitive {
                attributes: mesh_data.attributes.clone(),
                extensions: Default::default(),
                extras: Default::default(),
                indices: Some(mesh_data.indices),
                material: object_list.materials.get(&part.material_id).copied(),
                mode: Checked::Valid(mesh::Mode::Triangles),
                targets: None,
            }],
            weights: None,
        });

        // These variable names are from the original 3ds max import script.
        let part_scale = object_average_scale * (part.scale.x + part.scale.y + part.scale.z) / 3.0;
        let true_face_area = mesh_data.surface_area;
        let many_face_avt = (mesh_data.num_faces as f32) / 5000.0;
        let auto_rtt_map_size = (true_face_area + true_face_area * many_face_avt) * part_scale;
        let lightmap_size = if auto_rtt_map_size > 2500.0 {
            3 // 256
        } else if auto_rtt_map_size > 150.0 {
            2 // 128
        } else if auto_rtt_map_size > 11.0 {
            1 // 64
        } else {
            0 // 32
        };

        let node_index = Index::new(root.nodes.len() as u32);
        children.push(node_index);
        root.nodes.push(scene::Node {
            name: Some(format!(
                "{}_{}_{}_{}_{}",
                block.block_x, block.block_y, object_list_name, object_instance_index, part_index
            )),
            camera: None,
            children: None,
            extensions: Default::default(),
            extras: Some(
                RawValue::from_string(format!(
                    r#"{{
                        "TLM_ObjectProperties": {{
                            "tlm_mesh_lightmap_use": 1,
                            "tlm_mesh_lightmap_resolution": {},
                            "tlm_use_default_channel": 0,
                            "tlm_uv_channel": "UVMap.001"
                        }}
                    }}"#,
                    lightmap_size
                ))
                .unwrap(),
            ),
            matrix: None,
            mesh: Some(Index::new(mesh_index)),
            rotation: Some(convert_rotation(part.rotation)),
            scale: Some(convert_scale(part.scale)),
            translation: Some(convert_position(part.position)),
            skin: None,
            weights: None,
        });




        }
    }

    // Spawn a node for building object
    let node_index = Index::new(root.nodes.len() as u32);
    root.nodes.push(scene::Node {
        name: Some(format!(
            "{}_{}_{}_{}",
            block.block_x, block.block_y, object_list_name, object_instance_index,
        )),
        camera: None,
        children: Some(children),
        extensions: Default::default(),
        extras: Default::default(),
        matrix: None,
        mesh: None,
        rotation: Some(convert_rotation(object_instance.rotation)),
        scale: Some(convert_scale(object_instance.scale)),
        translation: Some(convert_position(object_instance.position)),
        skin: None,
        weights: None,
    });
    root.scenes[0].nodes.push(node_index);
}
