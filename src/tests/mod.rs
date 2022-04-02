#[cfg(test)]
mod tests {
    use crate::{Pixel, encode_from_pix, decode_to_pix};

    #[test]
    fn constant8x8() {
        let width = 8;
        let height = 8;
        let pixels = vec![Pixel::default(); width * height];
        let encoded = encode_from_pix(&pixels, width as u32, height as u32);

        let decoded = decode_to_pix(&encoded);

        assert_eq!(pixels, decoded);
    }

    #[test]
    fn increasing8x8() {
        let width = 8;
        let height = 8;
        let pixels = (0..width * height).map(|i| Pixel {
            r: i as u8,
            g: i as u8,
            b: i as u8,
            a: 255,
        }).collect::<Vec<_>>();
        let encoded = encode_from_pix(&pixels, width as u32, height as u32);

        let decoded = decode_to_pix(&encoded);

        assert_eq!(pixels, decoded);
    }

    #[test]
    fn random4x4() {
        let width = 4;
        let height = 4;
        let pixels = (0..width * height).map(|_| Pixel::random()).collect::<Vec<_>>();
        let encoded = encode_from_pix(&pixels, width as u32, height as u32);

        let decoded = decode_to_pix(&encoded);

        assert_eq!(pixels, decoded);
    }

    fn random(width: usize, height: usize) {
        let pixels = (0..width * height).map(|_| Pixel::random()).collect::<Vec<_>>();
        let encoded = encode_from_pix(&pixels, width as u32, height as u32);
        println!("encoded: {:?}", &encoded[14..]);

        let decoded = decode_to_pix(&encoded);
        println!("decoded: {:?}", decoded);

        assert_eq!(pixels, decoded);
    }

    #[test]
    fn repeated_random1x1() {
        loop {
            random(1, 1);
            println!("\n\n\n\n\n\n\n\n");
            println!("---------------------------------------------------------------------------------");
        }
    }
}
