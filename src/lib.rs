#![feature(mixed_integer_ops)]

mod structs;
pub use structs::*;

mod tests;

fn header(width: u32, height: u32, channels: Channels, colorspace: ColorSpace) -> [u8; 14] {
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

fn get_header(header: &[u8]) -> (u32, u32, Channels, ColorSpace) {
    let width = ((header[4] as u32) << 24)
        | ((header[5] as u32) << 16)
        | ((header[6] as u32) << 8)
        | (header[7] as u32);
    let height = ((header[8] as u32) << 24)
        | ((header[9] as u32) << 16)
        | ((header[10] as u32) << 8)
        | (header[11] as u32);
    let channels = match header[12] {
        3 => Channels::RGB,
        4 => Channels::RGBA,
        _ => panic!("Invalid number of channels"),
    };
    let colorspace = match header[13] {
        0 => ColorSpace::SRGB,
        1 => ColorSpace::Linear,
        _ => panic!("Invalid colorspace"),
    };
    (width, height, channels, colorspace)
}

pub fn encode_from_u8(pixels: &[u8], width: u32, height: u32) -> Vec<u8> {
    let pixels = pixels
        .chunks(3)
        .map(|chunk| {
            let mut pixel = Pixel::default();
            pixel.r = chunk[0];
            pixel.g = chunk[1];
            pixel.b = chunk[2];
            pixel
        })
        .collect::<Vec<_>>();
    encode_from_pix(&pixels, width, height)
}

pub fn encode_from_pix(pixels: &[Pixel], width: u32, height: u32) -> Vec<u8> {
    let mut hash = QOIHash::new();
    let mut encoded = Vec::from(header(width, height, Channels::RGB, ColorSpace::SRGB));
    let mut previous = Pixel::default();

    let num_pixels = pixels.len();
    assert_eq!(num_pixels, width as usize * height as usize);

    let mut i = 0;
    loop {
        if i >= num_pixels {
            break;
        }
        let pixel = pixels[i];
        let index = hash.lookup(&pixel);
        if previous == pixel {
            // start run of same color
            let mut num_same = 0; // start with offset, as spec
            loop {
                if num_same == 61 {
                    break;
                }
                if i + num_same + 1 < num_pixels && pixels[i + num_same + 1] == pixel {
                    num_same += 1;
                    hash.insert(&pixel);
                } else {
                    break;
                }
            }
            i = i + num_same + 1;
            previous = pixel;
            println!("found run of {} pixels", num_same + 1);
            println!("encoding as {}", OpRun::new(num_same as u8).get_encoding());
            encoded.push(OpRun::new(num_same as u8).get_encoding());
        } else if index.is_some() {
            // pixel exists in hash
            previous = pixel;
            println!("found pixel {:?} in hash", pixel);
            println!("encoding as {}", OpIndex::new(index.unwrap() as u8).get_encoding());
            encoded.push(OpIndex::new(index.unwrap() as u8).get_encoding());
            i += 1;
        } else {
            let dr = unsafe { std::mem::transmute::<_, i8>(pixel.r.wrapping_sub(previous.r)) };
            let dg = unsafe { std::mem::transmute::<_, i8>(pixel.g.wrapping_sub(previous.g)) };
            let db = unsafe { std::mem::transmute::<_, i8>(pixel.b.wrapping_sub(previous.b)) };
            if dr >= -2 && dr < 2 && dg >= -2 && dg < 2 && db >= -2 && db < 2 {
                // difference is small enough to be encoded with OpDiff
                previous = pixel;
                println!("found difference {:?}", (dr, dg, db));
                println!("encoding as {}", OpDiff::new(dr, dg, db).get_encoding());
                encoded.push(OpDiff::new(dr, dg, db).get_encoding());
            } else if dg >= -32
                && dg < 32
                && dr.wrapping_sub(dg) >= -8
                && dr.wrapping_sub(dg) < 8
                && db.wrapping_sub(dg) >= -8
                && db.wrapping_sub(dg) < 8
            {
                // difference is small enough to be encoded with OpLuma
                previous = pixel;
                println!("found large difference {:?}", (dr, dg, db));
                println!("encoding as {:?}", OpLuma::new(dr, dg, db).get_encoding());
                encoded.extend_from_slice(&OpLuma::new(dr, dg, db).get_encoding());
            } else {
                let pix_enc = OpRGB::new(pixel.r, pixel.g, pixel.b).get_encoding();
                previous = pixel;
                encoded.extend_from_slice(&pix_enc);
            }
            i += 1;
        }
    }
    encoded
}

pub fn decode_to_pix(encoded: &[u8]) -> Vec<Pixel> {
    let mut decoded = Vec::new();
    let mut hash = QOIHash::new();
    let mut previous = Pixel::default();

    let (width, height, channels, colorspace) = get_header(&encoded[0..14]);
    let encoded = &encoded[14..];

    // dbg!(&encoded);

    let mut i = 0; // which byte in encoded
    loop {
        if i >= encoded.len() {
            break;
        }
        let op = Chunk::from_encoding(&encoded[i..usize::min(i + 4, encoded.len())]);
        // dbg!(&op);
        match op {
            Chunk::RGB(rgb) => {
                let pixel = Pixel {
                    r: rgb.r,
                    g: rgb.g,
                    b: rgb.b,
                    a: 255,
                };
                hash.insert(&pixel);
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
                println!("rgba {:?}", rgba);
                hash.insert(&pixel);
                previous = pixel;
                decoded.push(pixel);
                i += 5;
            }
            Chunk::Index(index) => {
                let pixel = hash.get(index.index);
                println!("found pixel {:?} in hash from {}", pixel, &encoded[i]);
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
                println!("found difference {:?}", diff);
                hash.insert(&pixel);
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
                hash.insert(&pixel);
                previous = pixel;
                decoded.push(pixel);
                i += 2;
            }
            Chunk::Run(run) => {
                for _ in 0..=run.run {
                    decoded.push(previous);
                }
                i += 1;
            }
        }
    }
    decoded
}
