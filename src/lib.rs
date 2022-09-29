use image::{GenericImageView, Pixel, Rgba, ImageBuffer, SubImage};

type ImgRgba = ImageBuffer<Rgba<u8>, Vec<u8>>;

#[derive(Clone, Copy, Debug)]
pub enum Filter {
    Err,
    SqErr,
    Mse,
}

#[derive(Debug)]
struct Chunk {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    pixel: Rgba<u8>,
    error: u64,
    filter: Filter,
}

impl Chunk {
    fn from_img(img: &ImgRgba, x: u32, y: u32, width: u32, height: u32, filter: Filter) -> Self {
        //println!("Chunk::from_img(x: {x}, y: {y}, width: {width}, height: {height}");
        let sub = img.view(x, y, width, height);

        // Calculate raw error
        let calc = match filter {
            Filter::Err => err,
            Filter::SqErr => sq_err,
            Filter::Mse => mse,
        };
        let (error_raw, pixel) = calc(&sub);

        // Scale error based on color spectrum
        //let coeffs = [0.2989, 0.5870, 0.1140, 0.0];
        let coeffs = [1.0, 1.0, 1.0, 0.0];
        let error: u64 = std::iter::zip(error_raw, coeffs).map(|(e, c)| (e as f64 * c) as u64).sum();

        Self {
            x,
            y,
            width,
            height,
            pixel,
            error,
            filter,
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
            #[rustfmt::skip]
            let chunks = [
                Some(Chunk::from_img(img, x0, y0, width0, height0, self.filter)),
                Some(Chunk::from_img(img, x1, y0, width1, height0, self.filter)),
                Some(Chunk::from_img(img, x0, y1, width0, height1, self.filter)),
                Some(Chunk::from_img(img, x1, y1, width1, height1, self.filter)),
            ];
            chunks
        } else if self.width > 1 {
            // Chunk can only be split horizontally (vertical line)
            #[rustfmt::skip]
            let chunks = [
                Some(Chunk::from_img(img, x0, y0, width0, self.height, self.filter)),
                Some(Chunk::from_img(img, x1, y0, width1, self.height, self.filter)),
                None,
                None,
            ];
            chunks
        } else if self.height > 1 {
            // Chunk can only be split vertically (horizontal line)
            #[rustfmt::skip]
            let chunks = [
                Some(Chunk::from_img(img, x0, y0, self.width, height1, self.filter)),
                Some(Chunk::from_img(img, x0, y1, self.width, height1, self.filter)),
                None,
                None,
            ];
            chunks
        } else {
            // Not sure if this would really happen
            [None, None, None, None]
        }
    }
}

pub struct Quad<'a> {
    chunks: Vec<Chunk>,
    img: &'a ImgRgba,
}

impl<'a> Quad<'a> {
    pub fn from_img(img: &'a ImgRgba, depth: usize, filter: Filter) -> Self {
        let start = Chunk::from_img(&img, 0, 0, img.width(), img.height(), filter);
        let mut queue = vec![start];
        let mut count = 0;
        while let Some(chunk) = queue.pop() {
            // Get chunk with highest error

            // Split chunk into four new chunks (or fewer if unable to split)
            let chunks = chunk.split(img);

            // Put the new chunks into the queue. Keep the queue sorted.
            queue.extend(chunks.into_iter().filter_map(|x| x));
            queue.sort_unstable_by_key(|c| c.error); // TODO: use a BTreeSet?

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

    // Render each chunk into a new image
    pub fn render(&self, with_borders: bool) -> ImgRgba {
        let mut scratch = self.img.clone(); // TODO: clone img? or just create new one with same dims?
        for chunk in &self.chunks {
            let (x0, x1) = (chunk.x, chunk.x + chunk.width);
            let (y0, y1) = (chunk.y, chunk.y + chunk.height);

            for y in y0..y1 {
                for x in x0..x1 {
                    if with_borders == true && (x == x0 || x + 1 == x1 || y == y0 || y + 1 == y1) {
                        scratch.put_pixel(x, y, Rgba::from([0, 0, 0, 255]));
                    } else {
                        scratch.put_pixel(x, y, chunk.pixel);
                    }
                }
            }
        }
        scratch
    }
}

// Get average color
fn mean(sub: &SubImage<&ImgRgba>) -> Rgba<u8> {
    let mut total = [0, 0, 0, 0];
    for (_x, _y, p) in sub.pixels() {
        for i in 0..Rgba::<u8>::CHANNEL_COUNT as usize {
            total[i] += p[i] as u32;
        }
    }

    let mut mean = [0, 0, 0, 0];
    for i in 0..Rgba::<u8>::CHANNEL_COUNT as usize {
        mean[i] = (total[i] / sub.pixels().count() as u32).try_into().unwrap();
    }
    Rgba::from(mean)
}

// Calculate the total absolute error against a given pixel color
fn abs_err(sub: &SubImage<&ImgRgba>, pixel: Rgba<u8>) -> [u64; 4] {
    let mut output = [0u64; 4];
    for (_x, _y, p) in sub.pixels() {
        for i in 0..Rgba::<u8>::CHANNEL_COUNT as usize {
            output[i] += u8::abs_diff(p[i], pixel[i]) as u64;
        }
    }
    output
}

// Calculate the total absolute square error against a given pixel color
fn abs_err_sq(sub: &SubImage<&ImgRgba>, pixel: Rgba<u8>) -> [u64; 4] {
    let mut output = [0u64; 4];
    for (_x, _y, p) in sub.pixels() {
        for i in 0..Rgba::<u8>::CHANNEL_COUNT as usize {
            let err = u8::abs_diff(p[i], pixel[i]) as u64;
            output[i] += err * err;
        }
    }
    output
}

// Calculate total error
fn err(sub: &SubImage<&ImgRgba>) -> ([u64; 4], Rgba<u8>) {
    let mean = mean(sub);
    let output = abs_err(sub, mean);

    (output, Rgba::from(mean))
}

// Calculate total squared error
fn sq_err(sub: &SubImage<&ImgRgba>) -> ([u64; 4], Rgba<u8>) {
    let mean = mean(sub);
    let output = abs_err_sq(sub, mean);

    (output, Rgba::from(mean))
}

// Calculate mean squared error
fn mse(sub: &SubImage<&ImgRgba>) -> ([u64; 4], Rgba<u8>) {
    let (mut output, mean) = sq_err(sub);

    // MSE takes average of error
    for i in 0..Rgba::<u8>::CHANNEL_COUNT as usize {
        output[i] /= sub.pixels().count() as u64;
    }

    (output, Rgba::from(mean))
}
