use image::{io::Reader as ImageReader, ImageBuffer, Pixel};

struct Chunk {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

impl Chunk {
    fn split(self) -> [Self; 4] {
        #[rustfmt::skip]
        let chunks = [
            Chunk { x: self.x,                  y: self.y,                      width: self.width / 2,              height: self.height / 2                 },
            Chunk { x: self.x + self.width / 2, y: self.y,                      width: self.width - self.width / 2, height: self.height / 2                 },
            Chunk { x: self.x,                  y: self.y + self.height / 2,    width: self.width / 2,              height: self.height - self.height / 2   },
            Chunk { x: self.x + self.width / 2, y: self.y + self.height / 2,    width: self.width - self.width / 2, height: self.height - self.height / 2   },
        ];
        chunks
    }

    fn error<P: Pixel, C>(img: &ImageBuffer<P, C>) -> u32 {
        todo!()
    }
}

fn main() {
    let img = ImageReader::open("rock.jpg").unwrap().decode().unwrap();

    let mut scratch = img.clone();

    let chunk = Chunk { x: 0, y: 0, width: img.width(), height: img.height() };
    let mut queue = vec![chunk]; // TODO: make it a container of (error, chunk)
    while let Some(chunk) = queue.pop() {
        // Get chunk with highest error

        // Split chunk into four new chunks
        let chunks = chunk.split();

        // Filter each of the chunks into the scratch image
        for c in &chunks {
            // Get original image data for this chunk

            // Filter the data

            // Write this data into scratch
        }

        // Calculate the error for each chunk then put them back in the queue

    }

    scratch.save("output.png").unwrap();
}
