use clap::Parser;
use image::io::Reader as ImageReader;
use quadtree_image::{ErrCalc, Quad};
use std::path::PathBuf;

#[derive(Parser)]
struct Args {
    /// Input image filename
    input: PathBuf,
    /// Output filename [default: output.png]
    output: Option<PathBuf>,
    /// How many iterations to split the image [default: 500]
    depth: Option<u32>,
    /// Type of error calculation to use (linear, squared, mse) [default: squared]
    err_calc: Option<ErrCalc>,
}

fn main() {
    let args = Args::parse();

    let img = ImageReader::open(&args.input).unwrap().decode().unwrap();

    println!("Generating quadtree...");
    let quad = Quad::from_img(
        img,
        args.depth.unwrap_or(500),
        args.err_calc.unwrap_or(ErrCalc::SqErr),
    );
    println!("Rendering...");
    let output = quad.render(true);

    println!("Saving to disk...");
    output
        .save(&args.output.unwrap_or_else(|| "output.png".into()))
        .unwrap();
}
