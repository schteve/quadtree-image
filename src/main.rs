use image::io::Reader as ImageReader;
use quadtree_image::Quad;

fn main() {
    let img = ImageReader::open("rock.jpg").unwrap().decode().unwrap();
    let img = img.to_rgba8(); // To keep this program simple we only operate in RGBA space

    let quad = Quad::from_img(&img, 3); // TODO: let user specify depth
    let output = quad.render(true);

    output.save("output.png").unwrap();
}
