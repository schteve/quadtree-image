use clap::Parser;
use image::io::Reader as ImageReader;
use quadtree_image::{ErrCalc, Quad};
use std::path::PathBuf;

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
}

fn main() {
    let args = Args::parse();

    println!("Reading source image...");
    let img = ImageReader::open(&args.input).unwrap().decode().unwrap();

    println!("Generating quadtree...");
    let mut quad = Quad::from_img(img, args.err_calc);
    quad.process(args.depth);

    println!("Rendering...");
    let output = quad.render(!args.no_borders);

    println!("Saving to disk...");
    output.save(&args.output).unwrap();
}
