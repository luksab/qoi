use qoi::{decode_to_u8, encode_from_u8, QoiHeader};
use std::{
    fs::File,
    io::{BufWriter, Write},
};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt()]
    input: String,

    #[structopt()]
    output: String,
}

fn main() {
    let opt = Opt::from_args();
    println!("{:?}", opt);

    let file = File::open(&opt.input).expect("invalid input file");

    let (bytes, width, height) = {
        let png_decoder = png::Decoder::new(file);
        if let Ok(mut reader) = png_decoder.read_info() {
            println!("decoding png");
            // Allocate the output buffer.
            let mut buf = vec![0; reader.output_buffer_size()];
            // Read the next frame. An APNG might contain multiple frames.
            let info = reader.next_frame(&mut buf).unwrap();
            // Grab the bytes of the image.
            println!("{:?}", info);
            assert_eq!(info.bit_depth, png::BitDepth::Eight, "png bit depth must be 8");
            match info.color_type {
                png::ColorType::Grayscale => todo!(),
                png::ColorType::Rgb => {
                    let mut bytes = Vec::with_capacity((info.width * info.height * 4) as usize);
                    for row in buf.chunks(info.width as usize * 3) {
                        for pixel in row.chunks(3) {
                            bytes.push(pixel[0]);
                            bytes.push(pixel[1]);
                            bytes.push(pixel[2]);
                            bytes.push(255);
                        }
                    }
                    (bytes, info.width, info.height)
                }
                png::ColorType::Indexed => todo!(),
                png::ColorType::GrayscaleAlpha => todo!(),
                png::ColorType::Rgba => (buf, info.width, info.height),
            }
        } else {
            println!("decoding qoi");
            let bytes = std::fs::read(&opt.input).unwrap();
            let header = QoiHeader::from_u8(&bytes).expect("invalid qoi header");
            (decode_to_u8(&bytes), header.width, header.height)
        }
    };

    println!("{}x{}", width, height);

    match std::str::from_utf8(&opt.output.bytes().rev().take(3).rev().collect::<Vec<u8>>()).unwrap()
    {
        "png" => {
            println!("encoding png");
            let file = File::create(&opt.output).unwrap();
            let w = &mut BufWriter::new(file);
            let now = std::time::Instant::now();
            let mut encoder = png::Encoder::new(w, width as u32, height as u32);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder.write_header().unwrap();

            // let data = [255, 0, 0, 255, 0, 0, 0, 255]; // An array containing a RGBA sequence. First pixel is red and second pixel is black.
            writer.write_image_data(&bytes).unwrap(); // Save
            println!("encoded in {:?}", now.elapsed());
        }
        "qoi" => {
            println!("encoding qoi");
            let now = std::time::SystemTime::now();
            let encoded = encode_from_u8(&bytes, width, height); // save decoded to file
            println!("encoded in {:?}", now.elapsed().unwrap());
            let mut file = File::create(opt.output).unwrap();
            file.write_all(&encoded).unwrap();
        }
        _ => {
            panic!("invalid output format");
        }
    };
}
