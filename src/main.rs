use image::{io::Reader as ImageReader, DynamicImage, GenericImage, GenericImageView, Rgba};

struct Chunk {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    pixel: Rgba<u8>,
    error: u64,
}

impl Chunk {
    fn from_img(img: &DynamicImage, x: u32, y: u32, width: u32, height: u32) -> Self {
        let sub = img.view(x, y, width, height);

        // Filter the image data in the chunk's region
        // There's gotta be a better way
        let mut total = Rgba::<u32>::from([0, 0, 0, 0]);
        for (_x, _y, p) in sub.pixels() {
            for i in 0..4 {
                total[i] += p[i] as u32;
            }
        }

        let mut avg = Rgba::<u8>::from([0, 0, 0, 0]);
        for i in 0..4 {
            avg[i] = (total[i] / sub.pixels().count() as u32).try_into().unwrap();
        }
        let pixel = avg;

        // Calculate error
        let mut error: u64 = 0;
        for (_x, _y, p) in sub.pixels() {
            // Make an extension trait to handle this?
            for i in 0..4 {
                error += u8::abs_diff(p[i], pixel[i]) as u64;
            }
        }

        Self {
            x,
            y,
            width,
            height,
            pixel,
            error,
        }
    }

    fn split(self, img: &DynamicImage) -> [Self; 4] {
        let width0 = self.width / 2;
        let width1 = self.width - width0;
        let height0 = self.height / 2;
        let height1 = self.height - height0;
        let x0 = self.x;
        let x1 = self.x + width0;
        let y0 = self.y;
        let y1 = self.y + height0;

        #[rustfmt::skip]
        let chunks = [
            Chunk::from_img(img, x0, y0, width0, height0),
            Chunk::from_img(img, x1, y0, width1, height0),
            Chunk::from_img(img, x0, y1, width0, height1),
            Chunk::from_img(img, x1, y1, width1, height1),
        ];
        chunks
    }
}

fn main() {
    let img = ImageReader::open("rock.jpg").unwrap().decode().unwrap();

    let start = Chunk::from_img(&img, 0, 0, img.width(), img.height());
    let mut queue = vec![start];
    let mut count = 0;
    while let Some(chunk) = queue.pop() {
        // Get chunk with highest error

        // Split chunk into four new chunks
        let chunks = chunk.split(&img);

        // Put each chunk back in the queue
        queue.extend(chunks.into_iter());
        queue.sort_unstable_by_key(|c| c.error);

        count += 1;
        if count >= 100 {
            break;
        }
    }

    // Render each chunk into a new image
    let mut scratch = img;
    for (i, chunk) in queue.into_iter().enumerate() {
        println!("Chunk {i}");
        for y in chunk.y..chunk.y + chunk.height {
            for x in chunk.x..chunk.x + chunk.width {
                scratch.put_pixel(x, y, chunk.pixel);
            }
        }
    }
    scratch.save("output.png").unwrap();
}
