use qoi::{decode_to_pix, encode_from_pix, Pixel};
use rand::Rng;

fn cycling8x8() {
    let width = 8;
    let height = 8;

    let pixel_list = (0..10).map(|_| Pixel::random()).collect::<Vec<_>>();

    let pixels = (0..width * height)
        .map(|i| pixel_list[i % pixel_list.len()])
        .collect::<Vec<_>>();
    println!("{:?}", pixels);
    let encoded = encode_from_pix(&pixels, width as u32, height as u32);

    println!("encoded size: {}", encoded.len());
    println!("compression ratio: {}", encoded.len() as f32 / (width * height * 4) as f32);

    let decoded = decode_to_pix(&encoded);

    assert_eq!(pixels, decoded);
}

fn default8x8() {
    let width = 8;
    let height = 8;
    let pixels = vec![qoi::Pixel::default(); width * height];
    let encoded = qoi::encode_from_pix(&pixels, width as u32, height as u32);
    // println!("{:?}", encoded);

    let decoded = qoi::decode_to_pix(&encoded);
    // println!("{:?}", decoded);

    assert_eq!(pixels, decoded);
    println!("\n\nyay!\n");
}

fn random(width: usize, height: usize) {
    let pixels = (0..width * height)
        .map(|_| Pixel::random())
        .collect::<Vec<_>>();
    // println!("{:?}", pixels);
    let encoded = encode_from_pix(&pixels, width as u32, height as u32);
    // println!("encoded: {:?}", &encoded[14..]);

    let decoded = decode_to_pix(&encoded);
    // println!("decoded: {:?}", decoded);

    if pixels != decoded {
        println!("{:?}", pixels);
        println!("{:?}", encoded);
        println!("{:?}", decoded);
        panic!("pixels != decoded");
    }
    // assert_eq!(pixels, decoded);
}

fn repeated_random_x_by_x() {
    let mut rng = rand::thread_rng();
    for _ in 0..1_000_000 {
        random(rng.gen_range(1..100), rng.gen_range(1..100));
        // println!("\n\n\n\n\n\n\n\n");
        // println!(
        //     "---------------------------------------------------------------------------------"
        // );
    }
}

fn main() {
    cycling8x8();
    repeated_random_x_by_x();
}
