use std::fs::File;
use std::io::{BufRead, BufReader};

const PIXEL_TRANS: u8 = 2;

#[derive(Debug)]
struct Image {
    width: u32,
    height: u32,
    layers: u32,
    pixels: Vec<u8>,
}

impl Image {
    fn from_str(width: u32, height: u32, buf: &str) -> Image {
        let mut pixels: Vec<u8> = Vec::new();
        let layer_size = width * height;
        let layers: u32 = buf.len() as u32 / layer_size;

        let chars: Vec<char> = buf.chars().collect();

        for i in 0..width * height {
            let mut pixel: u8 = 0;
            for l in 0..layers {
                pixel = chars[(l * layer_size + i) as usize].to_digit(10).unwrap() as u8;
                if pixel != PIXEL_TRANS {
                    break;
                }
            }

            pixels.push(pixel);
        }

        return Image {
            width: width,
            height: height,
            layers: layers,
            pixels: pixels,
        };
    }

    fn from_file(width: u32, height: u32, filename: &str) -> Image {
        let file = File::open(filename).expect("Failed to open file");
        let mut reader = BufReader::new(file);
        let mut line = String::new();
        reader.read_line(&mut line).expect("Failed to read line");
        return Image::from_str(width, height, line.as_ref());
    }

    fn get_pixel_value(&self, x: u32, y: u32) -> u8 {
        return self.pixels[(x + self.width * y) as usize];
    }

    fn to_file(&self, filename: &str) {
        let mut buf = image::ImageBuffer::new(self.width, self.height);
        for (x, y, pixel) in buf.enumerate_pixels_mut() {
            let val = self.get_pixel_value(x, y) * 255;
            *pixel = image::Rgb([val, val, val]);
        }
        buf.save(filename).unwrap();
    }
}

fn main() {
    let img = Image::from_file(25, 6, "input");
    img.to_file("output.png");
}
