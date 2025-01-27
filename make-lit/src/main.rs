use std::{
    collections::HashMap,
    ffi::OsStr,
    hash::Hash,
    path::{Path, PathBuf},
    str::FromStr,
};

use clap::Parser;
use image::GenericImage;
use rose_file_lib::{
    files::{
        lit::{LightmapObject, LightmapPart},
        LIT,
    },
    io::RoseFile,
};

/// Converts lightmap textures to .LIT files
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Directoriy containing baked ligthmap textures, one file per object part.
    /// Filenames are expected to be of the format:
    ///   {block_x}.{block_y}.{object_type}.{object_id}.{part_id}[.*].png
    ///
    /// Where:
    ///   block_x, block_y is the .IFO block
    ///   object_type is either "deco" or "cnst"
    ///   object_id is the IFO object index
    ///   part_id is the ZSC object part index
    #[arg(short, long)]
    input: PathBuf,

    /// Directory to output generated lightmap files.
    #[arg(short, long)]
    output: PathBuf,

    /// Path to TexConv.exe for DDS generation.
    #[arg(short, long)]
    texconv: PathBuf,

    /// File extension for input images.
    #[arg(short, long, default_value = "exr")]
    extension: String,

    /// Separator used to parse input image file names.
    #[arg(short, long, default_value = ".")]
    separator: String,

    /// Texture width + height to use for lightmap atlas, must be divisible by 32, 64, 128, 256.
    #[arg(short = 'z', long, default_value_t = 512)]
    atlas_size: u32,

    /// Suppress output messages.
    #[arg(short, long, default_value_t = false)]
    quiet: bool,
}

#[derive(Clone, Copy, Debug)]
enum ObjectType {
    Deco,
    Cnst,
    Heightmap,
}

impl FromStr for ObjectType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "deco" => Ok(ObjectType::Deco),
            "cnst" => Ok(ObjectType::Cnst),
            _ => Err(()),
        }
    }
}

#[derive(Clone)]
struct LightmapImage {
    path: PathBuf,
    block_x: i32,
    block_y: i32,
    object_type: ObjectType,
    object_id: i32,
    part_id: i32,
}

fn parse_name(args: &Args, path: PathBuf) -> Option<LightmapImage> {
    if !path
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case(&args.extension))
    {
        return None;
    }

    let file_name = path.file_name()?.to_string_lossy();
    let split: Vec<&str> = file_name.split(&args.separator).collect();

    if file_name.contains("heightmap") {
        Some(LightmapImage {
            block_x: split.first()?.parse::<i32>().ok()?,
            // Ensure we ignore any trailing text after the final number
            block_y: split
                .get(1)?
                .chars()
                .take_while(|c| c.is_numeric())
                .collect::<String>()
                .parse::<i32>()
                .ok()?,
            object_type: ObjectType::Heightmap,
            object_id: 0,
            part_id: 0,
            path,
        })
    } else {
        Some(LightmapImage {
            block_x: split.first()?.parse::<i32>().ok()?,
            block_y: split.get(1)?.parse::<i32>().ok()?,
            object_type: split.get(2)?.parse::<ObjectType>().ok()?,
            object_id: split.get(3)?.parse::<i32>().ok()?,
            // Ensure we ignore any trailing text after the final number
            part_id: split
                .get(4)?
                .chars()
                .take_while(|c| c.is_numeric())
                .collect::<String>()
                .parse::<i32>()
                .ok()?,
            path,
        })
    }
}

fn collect_images(args: &Args, dir: &Path) -> Vec<LightmapImage> {
    let mut images = Vec::new();
    let Ok(iter) = std::fs::read_dir(dir) else {
        return images;
    };

    for entry in iter.flatten() {
        if let Some(image) = parse_name(args, entry.path()) {
            if !args.quiet {
                println!(
                    "Found image {} (block {}, {}, {:?}: {}, part: {})",
                    image.path.to_string_lossy(),
                    image.block_x,
                    image.block_y,
                    image.object_type,
                    image.object_id,
                    image.part_id
                );
            }
            images.push(image);
        }
    }

    images
}

#[derive(Hash, Eq, PartialEq, PartialOrd, Ord)]
struct ObjectId(pub i32);

#[derive(Hash, Eq, PartialEq, PartialOrd, Ord)]
struct PartId(pub i32);

type ObjectImages = HashMap<u32, HashMap<ObjectId, HashMap<PartId, image::RgbImage>>>;

struct LightmapBlock {
    block_x: i32,
    block_y: i32,
    cnst_by_size: ObjectImages,
    deco_by_size: ObjectImages,
    heightmap_image: image::RgbImage,
}

fn convert_hdr_image(input: image::Rgb32FImage) -> image::RgbImage {
    let mut dst = image::RgbImage::new(input.width(), input.height());
    for (src_pixel, dst_pixel) in input.pixels().zip(dst.pixels_mut()) {
        dst_pixel[0] = ((src_pixel[0] / 2.0) * 255.0).clamp(0.0, 255.0) as u8;
        dst_pixel[1] = ((src_pixel[1] / 2.0) * 255.0).clamp(0.0, 255.0) as u8;
        dst_pixel[2] = ((src_pixel[2] / 2.0) * 255.0).clamp(0.0, 255.0) as u8;
    }
    dst
}

fn main() {
    let args = Args::parse();
    let images = collect_images(&args, &args.input);

    // Collect image files into blocks -> object_type -> image size -> object_id -> part_id
    let mut blocks = HashMap::new();
    for lightmap_image in images.iter() {
        let image_data = image::open(&lightmap_image.path)
            .expect("Failed to load image")
            .to_rgb32f();

        if args.atlas_size % image_data.width() != 0 || args.atlas_size % image_data.height() != 0 {
            panic!("Image does not fit into atlas, invalid image size or invalid atlas size");
        }

        let block = blocks
            .entry((lightmap_image.block_x, lightmap_image.block_y))
            .or_insert_with(|| LightmapBlock {
                block_x: lightmap_image.block_x,
                block_y: lightmap_image.block_y,
                deco_by_size: HashMap::new(),
                cnst_by_size: HashMap::new(),
                heightmap_image: image::RgbImage::default(),
            });

        let size_map = match lightmap_image.object_type {
            ObjectType::Deco => &mut block.deco_by_size,
            ObjectType::Cnst => &mut block.cnst_by_size,
            ObjectType::Heightmap => {
                block.heightmap_image = convert_hdr_image(image_data);
                continue;
            }
        };

        let object_map = size_map
            .entry(image_data.width())
            .or_insert_with(HashMap::new);

        let part_map = object_map
            .entry(ObjectId(lightmap_image.object_id))
            .or_insert_with(HashMap::new);

        part_map.insert(
            PartId(lightmap_image.part_id),
            convert_hdr_image(image_data),
        );
    }

    let mut convert_to_dds = Vec::new();
    for block in blocks.values() {
        // Create directory for this block
        let block_directory = args
            .output
            .join(format!("{}_{}", block.block_x, block.block_y));
        let block_lightmap_directory = block_directory.join("LIGHTMAP");
        std::fs::create_dir_all(&block_lightmap_directory)
            .expect("Could not create block lightmap directory");

        generate_lightmaps(
            &args,
            "object",
            &block.deco_by_size,
            &block_lightmap_directory,
            &mut convert_to_dds,
        );

        generate_lightmaps(
            &args,
            "building",
            &block.cnst_by_size,
            &block_lightmap_directory,
            &mut convert_to_dds,
        );

        if !block.heightmap_image.is_empty() {
            let path = block_directory.join(format!(
                "{}_{}_planelightingmap.png",
                block.block_x, block.block_y
            ));
            block
                .heightmap_image
                .save_with_format(&path, image::ImageFormat::Png)
                .expect("Failed to write heightmap image");
            convert_to_dds.push(path);
        }
    }

    for path in convert_to_dds.iter() {
        if !args.quiet {
            println!("Converting to DDS {}", path.to_string_lossy());
        }

        std::process::Command::new(&args.texconv)
            .args([
                OsStr::new("-f"),      // Output format
                OsStr::new("DXT1"),    // DXT1 compression
                OsStr::new("-l"),      // Lowercase filenames
                OsStr::new("-y"),      // Overwrite existing files
                OsStr::new("-nologo"), // No stdout garbage
                OsStr::new("-m"),      // Generate mipmaps
                OsStr::new("1"),       // Number of mipmaps
                OsStr::new("-o"),      // Output directory
                path.parent().unwrap().as_os_str(),
                OsStr::new("--"),
                path.as_os_str(),
            ])
            .output()
            .expect("Failed to run texconv.exe");
    }
}

struct AtlasFile {
    name: String,
    image_data: image::RgbImage,
    columns: u32,
}

fn generate_lightmaps(
    args: &Args,
    group_name: &str,
    object_images: &ObjectImages,
    block_lightmap_directory: &Path,
    convert_to_dds: &mut Vec<PathBuf>,
) {
    let mut lit = LIT {
        objects: Vec::new(),
        filenames: Vec::new(),
    };

    for (&part_image_size, objects) in object_images.iter() {
        // Initialise the atlas files
        let mut atlas_files = Vec::<AtlasFile>::new();
        let atlas_max_columns = args.atlas_size / part_image_size;
        let atlas_max_rows = args.atlas_size / part_image_size;
        let atlas_max_parts = atlas_max_rows * atlas_max_columns;
        let num_parts: u32 = objects
            .iter()
            .map(|(_, object_parts)| object_parts.len() as u32)
            .sum();
        let num_atlas_files = num_parts.div_ceil(atlas_max_parts);

        for i in 0..num_atlas_files {
            let parts_in_atlas = (num_parts - i * atlas_max_parts).min(atlas_max_parts);

            // Atlas textures must be squares, find the smallest square that can contain parts_in_atlas
            let columns = (1..=atlas_max_columns)
                .find(|w| w * w >= parts_in_atlas)
                .unwrap_or(atlas_max_columns);
            let rows = columns;

            atlas_files.push(AtlasFile {
                name: format!("{}_{}_{}.dds", group_name, part_image_size, i),
                image_data: image::RgbImage::new(columns * part_image_size, rows * part_image_size),
                columns,
            });
        }

        // Add all parts to .LIT and copy image into atlas
        let mut atlas_file_index = 0;
        let mut atlas_part_index = 0;
        for (object_id, object_parts) in objects.iter() {
            let mut lit_parts = Vec::new();

            for (part_id, part_image) in object_parts.iter() {
                let atlas_file = &mut atlas_files[atlas_file_index];

                // Add to .LIT
                lit_parts.push(LightmapPart {
                    name: format!("{}_{}_{}", group_name, object_id.0, part_id.0),
                    id: part_id.0,
                    filename: atlas_file.name.clone(),
                    lightmap_index: atlas_file_index as i32,
                    pixels_per_part: part_image_size as i32,
                    parts_per_width: atlas_file.columns as i32,
                    part_position: atlas_part_index as i32,
                });

                // Copy image to atlas
                let x = (atlas_part_index % atlas_file.columns) * part_image_size;
                let y = (atlas_part_index / atlas_file.columns) * part_image_size;
                atlas_file
                    .image_data
                    .copy_from(part_image, x, y)
                    .expect("Failed to copy image into atlas");

                // Iterate through atlas
                atlas_part_index += 1;
                if atlas_part_index == atlas_max_parts {
                    atlas_part_index = 0;
                    atlas_file_index += 1;
                }
            }

            lit.objects.push(LightmapObject {
                id: object_id.0 + 1, // +1 as LIT objects are 1-indexed
                parts: lit_parts,
            });
        }

        // Write out all atlas files
        for atlas_file in atlas_files {
            let mut output_png = block_lightmap_directory.join(&atlas_file.name);
            output_png.set_extension("png"); // We can only encode to png, then convert to DDS after

            if !args.quiet {
                println!("Writing {}", output_png.to_string_lossy());
            }

            atlas_file
                .image_data
                .save_with_format(&output_png, image::ImageFormat::Png)
                .expect("Failed to write atlas png");
            convert_to_dds.push(output_png);
            lit.filenames.push(atlas_file.name);
        }
    }

    let lit_path = block_lightmap_directory.join(format!("{}lightmapdata.lit", group_name));
    if !args.quiet {
        println!("Writing {}", lit_path.to_string_lossy());
    }
    let mut file = std::fs::File::create(lit_path).expect("Failed to create LIT file");
    lit.write(&mut file).expect("Failed to write LIT file");
}
