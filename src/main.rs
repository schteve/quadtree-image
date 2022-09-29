use image::io::Reader as ImageReader;
use quadtree_image::{Quad, Filter};

fn main() {
    let img = ImageReader::open("rock.jpg").unwrap().decode().unwrap();
    let img = img.to_rgba8(); // To keep this program simple we only operate in RGBA space

    println!("Generating quadtree...");
    let quad = Quad::from_img(&img, 500, Filter::Mse); // TODO: let user specify depth
    println!("Rendering...");
    let output = quad.render(true);

    println!("Save to disk...");
    output.save("output.png").unwrap();
}
