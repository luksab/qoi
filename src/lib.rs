#![feature(mixed_integer_ops)]

mod structs;
use structs::*;

fn get_header(width: u32, height: u32, channels: Channels, colorspace: ColorSpace) -> [u8; 14] {
    let mut header = [0; 14];
    header[0] = "qoif".as_bytes()[0];
    header[1] = "qoif".as_bytes()[1];
    header[2] = "qoif".as_bytes()[2];
    header[3] = "qoif".as_bytes()[3];
    header[4] = (width >> 24) as u8;
    header[5] = (width >> 16) as u8;
    header[6] = (width >> 8) as u8;
    header[7] = width as u8;
    header[8] = (height >> 24) as u8;
    header[9] = (height >> 16) as u8;
    header[10] = (height >> 8) as u8;
    header[11] = height as u8;
    header[12] = channels as u8;
    header[13] = colorspace as u8;
    header
}

pub fn encode(pixels: &[Pixel], width: u32, height: u32) -> Vec<u8> {
    let mut hash = QOIHash::new();
    let mut encoded = Vec::from(get_header(width, height, Channels::RGB, ColorSpace::SRGB));
    let mut previous = Pixel::default();

    let num_pixels = pixels.len();
    assert_eq!(num_pixels, width as usize * height as usize);

    let mut i = 0;
    loop {
        if i >= num_pixels {
            break;
        }
        let pixel = pixels[i];
        if i + 1 < num_pixels && pixels[i + 1] == pixel {
            hash.insert(pixel);
            // start run of same color
            let mut num_same = 0; // start with offset, as spec
            loop {
                if num_same == 62 {
                    break;
                }
                if i + num_same + 1 < num_pixels && pixels[i + num_same + 1] == pixel {
                    num_same += 1;
                    hash.insert(pixel);
                } else {
                    break;
                }
            }
            i = i + num_same + 1;
            encoded.push(OpRun::new(num_same as u8).get_encoding());
        } else {
            let pix_enc = OpRGB::new(pixel.r, pixel.g, pixel.b).get_encoding();
            encoded.extend_from_slice(&pix_enc);
        }
    }
    encoded
}

pub fn decode(encoded: &[u8]) -> Vec<Pixel> {
    let mut decoded = Vec::new();
    let mut hash = QOIHash::new();
    let mut previous = Pixel::default();

    let mut i = 0; // which byte in encoded
    loop {
        if i >= encoded.len() {
            break;
        }
        let op = Chunk::from_encoding(&encoded[i..i + 4]);
        match op {
            Chunk::RGB(rgb) => {
                let pixel = Pixel {
                    r: rgb.r,
                    g: rgb.g,
                    b: rgb.b,
                    a: 255,
                };
                hash.insert(pixel);
                previous = pixel;
                decoded.push(pixel);
                i += 4;
            }
            Chunk::RGBA(rgba) => {
                let pixel = Pixel {
                    r: rgba.r,
                    g: rgba.g,
                    b: rgba.b,
                    a: rgba.a,
                };
                hash.insert(pixel);
                previous = pixel;
                decoded.push(pixel);
                i += 5;
            }
            Chunk::Index(index) => {
                let pixel = hash.get(index.index);
                // hash does not need to be updated
                previous = pixel;
                decoded.push(pixel);
                i += 1;
            }
            Chunk::Diff(diff) => {
                let diff = diff.get_diffs();
                let pixel = Pixel {
                    r: previous.r.overflowing_add_signed(diff.0).0,
                    g: previous.g.overflowing_add_signed(diff.1).0,
                    b: previous.b.overflowing_add_signed(diff.2).0,
                    a: previous.a,
                };
                hash.insert(pixel);
                previous = pixel;
                decoded.push(pixel);
                i += 1;
            }
            Chunk::Luma(luma) => {
                let diff = luma.get_diffs();
                let pixel = Pixel {
                    r: previous.r.overflowing_add_signed(diff.0).0,
                    g: previous.g.overflowing_add_signed(diff.1).0,
                    b: previous.b.overflowing_add_signed(diff.2).0,
                    a: previous.a,
                };
                hash.insert(pixel);
                previous = pixel;
                decoded.push(pixel);
                i += 1;
            }
            Chunk::Run(run) => {
                for _ in 0..run.run {
                    decoded.push(previous);
                }
                i += 1;
            }
        }
    }
    decoded
}
