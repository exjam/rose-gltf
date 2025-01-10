use std::path::PathBuf;

use anyhow::Context;
use clap::Parser;
use rose_gltf_lib::{
    gltf_to_rose, rose_to_gltf, save_gltf, GltfData, GltfFormat, GltfRoseConvOptions,
    RoseGltfConvOptions,
};

/// Converts ROSE files to a .gltf file
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// List of input files
    input: Vec<PathBuf>,

    /// Output file path
    #[arg(short, long = "out", default_value = ".")]
    output: PathBuf,

    /// When converting a zon, only use blocks with this x value.
    #[arg(long)]
    filter_block_x: Option<i32>,

    /// When converting a zon, only use blocks with this y value.
    #[arg(long)]
    filter_block_y: Option<i32>,

    /// Choose better triangulation for heightmaps, though it may not match your ROSE client.
    #[arg(long, default_value_t = true)]
    use_better_heightmap_triangles: bool,

    /// Ouput GLTF instead of GLB
    #[arg(long)]
    gltf: bool,

    /// When converting from GLTF to ZMO, this is the FPS to use for the generated ZMO.
    #[arg(short, long, default_value_t = 30)]
    zmo_fps: u32,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if args.input.iter().any(|x| {
        x.extension()
            .is_some_and(|extension| extension == "gltf" || extension == "glb")
    }) {
        // GLTF -> ROSE
        for input_file in &args.input {
            let (document, buffers, images) =
                gltf::import(input_file).expect("Failed to read GLTF file");
            let mut results = gltf_to_rose(
                &GltfData {
                    document,
                    buffers,
                    images,
                },
                &GltfRoseConvOptions {
                    zmo_fps: args.zmo_fps,
                },
            )?;
            results.save_to_dir(&args.output)?;
        }
    } else {
        // ROSE -> GLTF
        let gltf = rose_to_gltf(
            &args.input,
            &RoseGltfConvOptions {
                filter_block_x: args.filter_block_x,
                filter_block_y: args.filter_block_y,
                use_better_heightmap_triangles: args.use_better_heightmap_triangles,
            },
        )?;

        let format = if args.gltf {
            GltfFormat::Text
        } else {
            GltfFormat::Binary
        };

        let output = &args.output.with_extension(format.file_extension());
        save_gltf(&gltf, output, &format).context("Failed to save gltf")?;
    }

    Ok(())
}
