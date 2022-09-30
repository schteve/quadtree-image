use clap::Parser;
use image::io::Reader as ImageReader;
use image::{codecs::gif::GifEncoder, Frame};
use quadtree_image::{ErrCalc, Quad};
use std::{
    fs::File,
    path::{Path, PathBuf},
};

#[derive(Parser)]
struct Args {
    /// Input image filename
    input: PathBuf,
    /// Output filename
    #[arg(short, long, default_value = "output.png")]
    output: PathBuf,
    /// How many times to split the image
    #[arg(short, long, default_value_t = 500)]
    depth: u32,
    /// Don't show borders
    #[arg(short, long)]
    no_borders: bool,
    /// Type of error calculation to use
    #[arg(short, long, value_enum, default_value_t = ErrCalc::Square)]
    err_calc: ErrCalc,
    /// Output an animation instead of a single image
    #[arg(short, long)]
    animation: bool,
}

fn main() {
    let args = Args::parse();

    println!("Reading source image...");
    let img = ImageReader::open(&args.input).unwrap().decode().unwrap();

    println!("Generating quadtree...");
    let mut quad = Quad::from_img(img, args.err_calc);
    if args.animation {
        println!("Encoding gif (this might take a while)...");
        let out_file_name = if args.output.to_string_lossy() == "output.png" {
            Path::new("output.gif")
        } else {
            &args.output
        };
        let out_file = File::create(out_file_name).unwrap();

        let frames = (0..args.depth).map(|_| {
            quad.process(1);
            let rendered = quad.render(!args.no_borders);
            Frame::new(rendered)
        });
        GifEncoder::new(out_file).encode_frames(frames).unwrap();
    } else {
        quad.process(args.depth);

        println!("Rendering...");
        let output = quad.render(!args.no_borders);

        println!("Saving to disk...");
        output.save(&args.output).unwrap();
    }
}
