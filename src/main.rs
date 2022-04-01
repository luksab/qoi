fn main() {
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
