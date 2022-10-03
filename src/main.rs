use clap::Parser;
use image::imageops::FilterType;
use image::io::Reader as ImageReader;
use image::{
    codecs::gif::{GifEncoder, Repeat},
    Frame,
};
use png::{BitDepth, ColorType, Encoder};
use quadtree_image::{ErrCalc, Quad};
use std::{fs::File, path::PathBuf};

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
    /// Loop the animation
    #[arg(short, long)]
    loop_anim: bool,
    /// Resize the image to 480x480 (or smaller) before processing. This is most
    /// useful for gifs, which can take a long time to encode if they are large.
    #[arg(short, long)]
    resize: bool,
}

fn main() {
    let args = Args::parse();

    println!("Reading source image...");
    let mut img = ImageReader::open(&args.input).unwrap().decode().unwrap();

    if args.resize {
        println!("Resizing source image...");
        img = img.resize(480, 480, FilterType::Lanczos3);
    }
    let (width, height) = (img.width(), img.height());

    println!("Generating quadtree...");
    let mut quad = Quad::from_img(img, args.err_calc);
    if args.animation {
        println!("Encoding gif... (this might take a while)");
        let out_file = File::create(&args.output).unwrap();

        let frames_iter = (0..args.depth).map(|i| {
            eprint!("\r    Frame {} / {}", i + 1, args.depth);
            quad.process(1);
            quad.render(!args.no_borders)
        });

        match args.output.extension() {
            Some(x) if x == "png" => {
                let mut encoder = Encoder::new(out_file, width, height);
                encoder.set_color(ColorType::Rgba);
                encoder.set_depth(BitDepth::Eight);
                let num_plays = if args.loop_anim { 0 } else { 1 };
                encoder.set_animated(args.depth, num_plays).unwrap();
                encoder.validate_sequence(true);
                let mut writer = encoder.write_header().unwrap();

                for frame in frames_iter {
                    writer.write_image_data(&frame.into_raw()).unwrap();
                }
                writer.finish().unwrap();
            }
            Some(x) if x == "gif" => {
                let mut encoder = GifEncoder::new(out_file);
                if args.loop_anim {
                    encoder.set_repeat(Repeat::Infinite).unwrap();
                }

                for frame in frames_iter {
                    // Note: this line is where all the slowness happens. Seems that the `image` crate uses
                    // the `gif` crate with an expensive computation to determine the palette to use for each
                    // frame. After hitting 256 colors per frame it really chugs. Using another crate like
                    // `libimagequant` to determine a fixed palette for all frames doesn't seem to help.
                    let f = Frame::new(frame);
                    encoder.encode_frame(f).unwrap();
                }
            }
            Some(x) => {
                panic!(
                    "Unsupported animation style requested: {:?}",
                    x.to_string_lossy()
                );
            }
            _ => panic!("No animation style requested (use the file extension)"),
        }
        eprintln!();
    } else {
        quad.process(args.depth);

        println!("Rendering...");
        let output = quad.render(!args.no_borders);

        println!("Saving to disk...");
        output.save(&args.output).unwrap();
    }

    println!("Complete!");
}
