use clap::ValueEnum;
use image::{DynamicImage, GenericImageView, ImageBuffer, Pixel, Rgba, SubImage};
use std::iter;

type ImgRgba = ImageBuffer<Rgba<u8>, Vec<u8>>;

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum ErrCalc {
    Linear,
    SqErr,
    Mse,
}

#[derive(Debug)]
struct Chunk {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    color: Rgba<u8>,
    error: u64,
}

impl Chunk {
    fn from_img(img: &ImgRgba, x: u32, y: u32, width: u32, height: u32, err_calc: ErrCalc) -> Self {
        //println!("Chunk::from_img(x: {x}, y: {y}, width: {width}, height: {height}");
        let sub = img.view(x, y, width, height);

        // Calculate raw error
        let calc = match err_calc {
            ErrCalc::Linear => linear_err,
            ErrCalc::SqErr => sq_err,
            ErrCalc::Mse => mse,
        };
        let (error_raw, color) = calc(&sub);

        // Scale error based on color spectrum
        //let coeffs = [0.2989, 0.5870, 0.1140, 0.0];
        let coeffs = [1.0, 1.0, 1.0, 0.0];
        let error: u64 = iter::zip(error_raw, coeffs)
            .map(|(e, c)| (e as f32 * c) as u64)
            .sum();

        Self {
            x,
            y,
            width,
            height,
            color,
            error,
        }
    }

    fn split(self, img: &ImgRgba, err_calc: ErrCalc) -> [Option<Self>; 4] {
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
                Some(Chunk::from_img(img, x0, y0, width0, height0, err_calc)),
                Some(Chunk::from_img(img, x1, y0, width1, height0, err_calc)),
                Some(Chunk::from_img(img, x0, y1, width0, height1, err_calc)),
                Some(Chunk::from_img(img, x1, y1, width1, height1, err_calc)),
            ];
            chunks
        } else if self.width > 1 {
            // Chunk can only be split horizontally (vertical line)
            #[rustfmt::skip]
            let chunks = [
                Some(Chunk::from_img(img, x0, y0, width0, self.height, err_calc)),
                Some(Chunk::from_img(img, x1, y0, width1, self.height, err_calc)),
                None,
                None,
            ];
            chunks
        } else if self.height > 1 {
            // Chunk can only be split vertically (horizontal line)
            #[rustfmt::skip]
            let chunks = [
                Some(Chunk::from_img(img, x0, y0, self.width, height1, err_calc)),
                Some(Chunk::from_img(img, x0, y1, self.width, height1, err_calc)),
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

pub struct Quad {
    chunks: Vec<Chunk>,
    img: ImgRgba,
    err_calc: ErrCalc,
}

impl Quad {
    #[must_use]
    pub fn from_img(img: DynamicImage, err_calc: ErrCalc) -> Self {
        let img = img.into_rgba8(); // To keep this program simple we only operate in RGBA space
        let start = Chunk::from_img(&img, 0, 0, img.width(), img.height(), err_calc);
        Self {
            chunks: vec![start],
            img,
            err_calc,
        }
    }

    pub fn process(&mut self, depth: u32) {
        for _ in 0..depth {
            if let Some(chunk) = self.chunks.pop() {
                // Get chunk with highest error

                // Split chunk into four new chunks (or fewer if unable to split)
                let chunks = chunk.split(&self.img, self.err_calc);

                // Put the new chunks into the queue. Keep the queue sorted.
                self.chunks.extend(chunks.into_iter().flatten());
                self.chunks.sort_unstable_by_key(|c| c.error);
            } else {
                // Didn't get to the specified depth but ran out of chunks to process
                break;
            }
        }
    }

    // Render each chunk into a new image
    #[must_use]
    pub fn render(&self, with_borders: bool) -> ImgRgba {
        let mut scratch = ImgRgba::new(self.img.width(), self.img.height());
        for chunk in &self.chunks {
            let (x0, x1) = (chunk.x, chunk.x + chunk.width);
            let (y0, y1) = (chunk.y, chunk.y + chunk.height);

            for y in y0..y1 {
                for x in x0..x1 {
                    if with_borders && (x == x0 || x + 1 == x1 || y == y0 || y + 1 == y1) {
                        scratch.put_pixel(x, y, [0, 0, 0, 255].into());
                    } else {
                        scratch.put_pixel(x, y, chunk.color);
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
        for (i, t) in total.iter_mut().enumerate() {
            let x: u32 = p[i].into();
            *t += x;
        }
    }

    let mut mean = [0, 0, 0, 0];
    let count: u32 = sub.pixels().count().try_into().unwrap();
    for (m, t) in iter::zip(mean.iter_mut(), total.iter()) {
        *m = (t / count).try_into().unwrap();
    }
    mean.into()
}

// Calculate the total absolute error against a given pixel color
fn abs_err(sub: &SubImage<&ImgRgba>, base: Rgba<u8>) -> [u64; 4] {
    let mut output = [0u64; 4];
    for (_x, _y, p) in sub.pixels() {
        for (i, o) in output.iter_mut().enumerate() {
            let diff: u64 = u8::abs_diff(p[i], base[i]).into();
            *o += diff;
        }
    }
    output
}

// Calculate the total absolute square error against a given pixel color
fn abs_err_sq(sub: &SubImage<&ImgRgba>, base: Rgba<u8>) -> [u64; 4] {
    let mut output = [0u64; 4];
    for (_x, _y, p) in sub.pixels() {
        for i in 0..Rgba::<u8>::CHANNEL_COUNT as usize {
            let err: u64 = u8::abs_diff(p[i], base[i]).into();
            output[i] += err * err;
        }
    }
    output
}

// Calculate total error
fn linear_err(sub: &SubImage<&ImgRgba>) -> ([u64; 4], Rgba<u8>) {
    let mean = mean(sub);
    let output = abs_err(sub, mean);

    (output, mean)
}

// Calculate total squared error
fn sq_err(sub: &SubImage<&ImgRgba>) -> ([u64; 4], Rgba<u8>) {
    let mean = mean(sub);
    let output = abs_err_sq(sub, mean);

    (output, mean)
}

// Calculate mean squared error
fn mse(sub: &SubImage<&ImgRgba>) -> ([u64; 4], Rgba<u8>) {
    let (mut output, mean) = sq_err(sub);

    // MSE takes average of error
    let count = sub.pixels().count() as u64;
    for o in &mut output {
        *o /= count;
    }

    (output, mean)
}
