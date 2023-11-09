use std::{io::Cursor, path::PathBuf};

use bytes::{BufMut, BytesMut};
use glam::{EulerRot, Quat, Vec2, Vec3};
use gltf_json::{
    buffer,
    extensions::{
        self,
        scene::khr_lights_punctual::{KhrLightsPunctual, Light},
    },
    material, mesh,
    scene::{self, UnitQuaternion},
    texture,
    validation::Checked,
    Index,
};
use roselib::{
    files::{him::Heightmap, ifo::MapData, til::Tilemap, zon, HIM, IFO, TIL},
    io::RoseFile,
};
use serde_json::{json, value::RawValue};

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

fn convert_rotation(rotation: roselib::utils::Quaternion) -> UnitQuaternion {
    UnitQuaternion([rotation.x, rotation.z, -rotation.y, rotation.w])
}

fn generate_terrain_materials(
    root: &mut gltf_json::Root,
    binary_data: &mut BytesMut,
    zon: &zon::Zone,
    assets_path: PathBuf,
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
            byte_length: texture_data_length,
            byte_offset: Some(texture_data_start),
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

pub fn load_zone(
    root: &mut gltf_json::Root,
    binary_data: &mut BytesMut,
    zon: &zon::Zone,
    assets_path: PathBuf,
    map_path: PathBuf,
    deco: &mut ObjectList,
    cnst: &mut ObjectList,
    filter_block_x: Option<i32>,
    filter_block_y: Option<i32>,
) {
    // Add a directional light to the scene
    root.extensions_used.push("KHR_lights_punctual".to_string());
    root.extensions = Some(extensions::Root {
        khr_lights_punctual: Some(extensions::root::KhrLightsPunctual {
            lights: vec![Light {
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

    // Load all meshes and materials from used objects
    for block in blocks.iter() {
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
        generate_terrain_materials(root, binary_data, zon, assets_path, &blocks);

    // Spawn all block nodes
    for (block, block_terrain_material) in blocks.iter().zip(block_terrain_materials.iter()) {
        // Create heightmap mesh
        {
            let mesh_data = generate_terrain_mesh(root, binary_data, block);

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

        // Spawn all object nodes
        for (object_instance_index, object_instance) in block.ifo.objects.iter().enumerate() {
            let mut children = Vec::new();
            let object_id = object_instance.object_id as usize;
            let object = &deco.zsc.objects[object_id];
            let object_scale =
                (object_instance.scale.x + object_instance.scale.y + object_instance.scale.z) / 3.0;

            // Spawn a node for each object part
            for (part_index, part) in object.parts.iter().enumerate() {
                let mesh_data = deco.meshes.get(&part.mesh_id).expect("Missing mesh");
                let mesh_index = root.meshes.len() as u32;
                root.meshes.push(mesh::Mesh {
                    name: Some(format!(
                        "{}_{}_deco_{}_{}_mesh",
                        block.block_x, block.block_y, object_instance_index, part_index
                    )),
                    extensions: Default::default(),
                    extras: Default::default(),
                    primitives: vec![mesh::Primitive {
                        attributes: mesh_data.attributes.clone(),
                        extensions: Default::default(),
                        extras: Default::default(),
                        indices: Some(mesh_data.indices),
                        material: deco.materials.get(&part.material_id).copied(),
                        mode: Checked::Valid(mesh::Mode::Triangles),
                        targets: None,
                    }],
                    weights: None,
                });

                // These variable names are from the original 3ds max import script.
                let part_scale = object_scale * (part.scale.x + part.scale.y + part.scale.z) / 3.0;
                let true_face_area = mesh_data.surface_area;
                let many_face_avt = (mesh_data.num_faces as f32) / 5000.0;
                let auto_rtt_map_size =
                    (true_face_area + true_face_area * many_face_avt) * part_scale;
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
                root.nodes.push(scene::Node {
                    name: Some(format!(
                        "{}_{}_deco_{}_{}_",
                        block.block_x, block.block_y, object_instance_index, part_index
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
                    scale: Some([part.scale.x, part.scale.y, part.scale.z]),
                    translation: Some(convert_position(part.position)),
                    skin: None,
                    weights: None,
                });
                children.push(node_index);
            }

            // Spawn a node for a object
            let node_index = Index::new(root.nodes.len() as u32);
            root.nodes.push(scene::Node {
                camera: None,
                children: Some(children),
                extensions: Default::default(),
                extras: Default::default(),
                matrix: None,
                mesh: None,
                name: Some(format!(
                    "{}_{}_deco_{}",
                    block.block_x, block.block_y, object_instance_index,
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
            root.scenes[0].nodes.push(node_index);
        }

        // Spawn a node for each building part
        for (object_instance_index, object_instance) in block.ifo.buildings.iter().enumerate() {
            let mut children = Vec::new();
            let object_id = object_instance.object_id as usize;
            let object = &cnst.zsc.objects[object_id];
            let object_scale =
                (object_instance.scale.x + object_instance.scale.y + object_instance.scale.z) / 3.0;

            // Spawn a node for each object part
            for (part_index, part) in object.parts.iter().enumerate() {
                let mesh_data = cnst.meshes.get(&part.mesh_id).expect("Missing mesh");
                let mesh_index = root.meshes.len() as u32;
                root.meshes.push(mesh::Mesh {
                    name: Some(format!(
                        "{}_{}_cnst_{}_{}_mesh",
                        block.block_x, block.block_y, object_instance_index, part_index
                    )),
                    extensions: Default::default(),
                    extras: Default::default(),
                    primitives: vec![mesh::Primitive {
                        attributes: mesh_data.attributes.clone(),
                        extensions: Default::default(),
                        extras: Default::default(),
                        indices: Some(mesh_data.indices),
                        material: cnst.materials.get(&part.material_id).copied(),
                        mode: Checked::Valid(mesh::Mode::Triangles),
                        targets: None,
                    }],
                    weights: None,
                });

                // These variable names are from the original 3ds max import script.
                let part_scale = object_scale * (part.scale.x + part.scale.y + part.scale.z) / 3.0;
                let true_face_area = mesh_data.surface_area;
                let many_face_avt = (mesh_data.num_faces as f32) / 5000.0;
                let auto_rtt_map_size =
                    (true_face_area + true_face_area * many_face_avt) * part_scale;
                let lightmap_size = if auto_rtt_map_size > 2500.0 {
                    3 // 256
                } else if auto_rtt_map_size > 150.0 {
                    2 // 128
                } else if auto_rtt_map_size > 11.0 {
                    1 // 64
                } else {
                    0 // 32
                };

                children.push(Index::new(root.nodes.len() as u32));
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
                            lightmap_size
                        ))
                        .unwrap(),
                    ),
                    matrix: None,
                    mesh: Some(Index::new(mesh_index)),
                    name: Some(format!(
                        "{}_{}_cnst_{}_{}",
                        block.block_x, block.block_y, object_instance_index, part_index
                    )),
                    rotation: Some(convert_rotation(part.rotation)),
                    scale: Some([part.scale.x, part.scale.y, part.scale.z]),
                    translation: Some(convert_position(part.position)),
                    skin: None,
                    weights: None,
                });
            }

            // Spawn a node for building object
            let node_index = Index::new(root.nodes.len() as u32);
            root.nodes.push(scene::Node {
                camera: None,
                children: Some(children),
                extensions: Default::default(),
                extras: Default::default(),
                matrix: None,
                mesh: None,
                name: Some(format!(
                    "{}_{}_cnst_{}",
                    block.block_x, block.block_y, object_instance_index,
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
            root.scenes[0].nodes.push(node_index);
        }
    }
}
