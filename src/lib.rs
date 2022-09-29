use image::{GenericImageView, Pixel, Rgba, ImageBuffer};

type ImgRgba = ImageBuffer<Rgba<u8>, Vec<u8>>;

#[derive(Debug)]
struct Chunk {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    pixel: Rgba<u8>,
    error: u64,
}

impl Chunk {
    fn from_img(img: &ImgRgba, x: u32, y: u32, width: u32, height: u32) -> Self {
        println!("Chunk::from_img(x: {x}, y: {y}, width: {width}, height: {height}");
        let sub = img.view(x, y, width, height);

        // Filter the image data in the chunk's region
        // There's gotta be a better way
        let mut total = [0, 0, 0, 0];
        for (_x, _y, p) in sub.pixels() {
            for i in 0..Rgba::<u8>::CHANNEL_COUNT as usize {
                total[i] += p[i] as u32;
            }
        }

        let mut avg = [0, 0, 0, 0];
        for i in 0..Rgba::<u8>::CHANNEL_COUNT as usize {
            avg[i] = (total[i] / sub.pixels().count() as u32).try_into().unwrap();
        }

        // Calculate error
        let mut mse = [0u64; 4];
        for (_x, _y, p) in sub.pixels() {
            // Make an extension trait to handle this?
            for i in 0..Rgba::<u8>::CHANNEL_COUNT as usize {
                let err = u8::abs_diff(p[i], avg[i]) as u64;
                mse[i] += err * err;
            }
        }
        for i in 0..Rgba::<u8>::CHANNEL_COUNT as usize {
            mse[i] /= sub.pixels().count() as u64;
        }

        //let coeffs = [0.2989, 0.5870, 0.1140, 0.0];
        let coeffs = [1.0, 1.0, 1.0, 1.0];
        let error: u64 = std::iter::zip(mse, coeffs).map(|(e, c)| (e as f64 * c) as u64).sum();

        Self {
            x,
            y,
            width,
            height,
            pixel: Rgba::from(avg),
            error,
        }
    }

    fn split(self, img: &ImgRgba) -> [Option<Self>; 4] {
        let width0 = self.width / 2;
        let width1 = self.width - width0;
        let height0 = self.height / 2;
        let height1 = self.height - height0;
        let x0 = self.x;
        let x1 = self.x + width0;
        let y0 = self.y;
        let y1 = self.y + height0;

        if self.width > 1 && self.height > 1 {
            // Chunk is big enough to split into four
            println!("Split 4");
            #[rustfmt::skip]
            let chunks = [
                Some(Chunk::from_img(img, x0, y0, width0, height0)),
                Some(Chunk::from_img(img, x1, y0, width1, height0)),
                Some(Chunk::from_img(img, x0, y1, width0, height1)),
                Some(Chunk::from_img(img, x1, y1, width1, height1)),
            ];
            chunks
        } else if self.width > 1 {
            // Chunk can only be split horizontally (vertical line)
            println!("Split horiz");
            #[rustfmt::skip]
            let chunks = [
                Some(Chunk::from_img(img, x0, y0, width0, self.height)),
                Some(Chunk::from_img(img, x1, y0, width1, self.height)),
                None,
                None,
            ];
            chunks
        } else if self.height > 1 {
            // Chunk can only be split vertically (horizontal line)
            println!("Split vert");
            #[rustfmt::skip]
            let chunks = [
                Some(Chunk::from_img(img, x0, y0, self.width, height1)),
                Some(Chunk::from_img(img, x0, y1, self.width, height1)),
                None,
                None,
            ];
            chunks
        } else {
            // Not sure if this would really happen
            println!("Split none");
            [None, None, None, None]
        }
    }
}

pub struct Quad<'a> {
    chunks: Vec<Chunk>,
    img: &'a ImgRgba,
}

impl<'a> Quad<'a> {
    pub fn from_img(img: &'a ImgRgba, depth: usize) -> Self {
        let start = Chunk::from_img(&img, 0, 0, img.width(), img.height());
        let mut queue = vec![start];
        let mut count = 0;
        while let Some(chunk) = queue.pop() {
            // Get chunk with highest error
            println!("Error: {}", chunk.error);

            // Split chunk into four new chunks ()
            let chunks = chunk.split(img);

            // Put each chunk back in the queue
            queue.extend(chunks.into_iter().filter_map(|x| x));
            queue.sort_unstable_by_key(|c| c.error);
            //dbg!(&queue);

            count += 1;
            if count >= depth {
                break;
            }
        }

        Self {
            chunks: queue,
            img,
        }
    }

    pub fn render(&self, with_borders: bool) -> ImgRgba {
        // Render each chunk into a new image
        let mut scratch = self.img.clone(); // TODO: clone img? or just create new one with same dims?
        for chunk in &self.chunks {
            let (x0, x1) = (chunk.x, chunk.x + chunk.width);
            let (y0, y1) = (chunk.y, chunk.y + chunk.height);

            for y in y0..y1 {
                for x in x0..x1 {
                    if with_borders == true && (x == x0 || x + 1 == x1 || y == y0 || y + 1 == y1) {
                        scratch.put_pixel(x, y, Rgba::from([0, 0, 0, 0]));
                    } else {
                        scratch.put_pixel(x, y, chunk.pixel);
                    }
                }
            }
        }
        scratch
    }
}
