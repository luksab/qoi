fn main() {
    let width = 8;
    let height = 8;
    let pixels = vec![qoi::Pixel::default(); width * height];
    let encoded = qoi::encode(&pixels, width as u32, height as u32);
    // println!("{:?}", encoded);

    let decoded = qoi::decode(&encoded);
    // println!("{:?}", decoded);

    assert_eq!(pixels, decoded);
    println!("\n\nyay!\n");
}
