#[repr(u8)]
enum Channels {
    RGB = 3,
    RGBA = 4,
}

enum ColorSpace {
    SRGB = 0,
    Linear = 1,
}

struct QoiHeader {
    magic: [u8; 4],
    width: u32,
    height: u32,
    channels: Channels,
    color_space: ColorSpace,
}

#[derive(Debug, Clone, Copy)]
struct Pixel {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Default for Pixel {
    fn default() -> Self {
        Pixel {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        }
    }
}

struct hash {
    data: Box<[Pixel]>,
}

impl hash {
    pub fn new() -> Self {
        hash {
            data: vec![Pixel::default(); 64].into_boxed_slice(),
        }
    }

    fn get_pos(&self, pixel: Pixel) -> usize {
        ((pixel.r * 3 + pixel.g * 5 + pixel.b * 11) % 64) as usize
    }
}

struct OpRGB {
    r: u8,
    g: u8,
    b: u8,
}

impl OpRGB {
    fn new(r: u8, g: u8, b: u8) -> Self {
        OpRGB { r, g, b }
    }

    fn get_tag(&self) -> u8 {
        0b11111110
    }

    pub fn get_encoding(&self) -> [u8; 4] {
        [self.get_tag(), self.r, self.g, self.b]
    }
}

struct OpRGBA {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl OpRGBA {
    fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        OpRGBA { r, g, b, a }
    }

    fn get_tag(&self) -> u8 {
        0b11111111
    }

    pub fn get_encoding(&self) -> [u8; 5] {
        [self.get_tag(), self.r, self.g, self.b, self.a]
    }
}

struct OpIndex {
    index: u8,
}

impl OpIndex {
    fn new(index: u8) -> Self {
        debug_assert!(index < (1 << 6));
        OpIndex { index }
    }

    fn get_tag(&self) -> u8 {
        0b00
    }

    pub fn get_encoding(&self) -> u8 {
        self.get_tag() | self.index
    }
}

struct OpDiff {
    diff: u8,
}

impl OpDiff {
    fn new(r: i8, g: i8, b: i8) -> Self {
        debug_assert!(r >= -2 && r <= 1);
        debug_assert!(g >= -2 && g <= 1);
        debug_assert!(b >= -2 && b <= 1);
        OpDiff {
            diff: ((r as u8) << 4) | ((g as u8) << 2) | (b as u8),
        }
    }

    fn get_tag(&self) -> u8 {
        0b01
    }

    pub fn get_encoding(&self) -> u8 {
        self.diff | (self.get_tag() << 6)
    }
}

struct OpLuma {
    g: u8,
    rb: u8,
}

impl OpLuma {
    fn new(dr: i8, dg: i8, db: i8) -> Self {
        debug_assert!(dg >= -32 && dg <= 31);
        debug_assert!(dr - dg >= -8 && dr - dg <= 7);
        debug_assert!(db - dg >= -8 && db - dg <= 7);
        OpLuma {
            g: unsafe { std::mem::transmute::<_, u8>(dg) },
            rb: (unsafe { std::mem::transmute::<_, u8>(dr - dg) } << 4)
                | unsafe { std::mem::transmute::<_, u8>(db - dg) },
        }
    }

    fn get_tag(&self) -> u8 {
        0b10
    }

    pub fn get_encoding(&self) -> [u8; 2] {
        [self.get_tag() << 6 | self.g, self.rb]
    }
}

struct OpRun {
    run: u8,
}

impl OpRun {
    fn new(run: u8) -> Self {
        debug_assert!(run < (1 << 6));
        OpRun { run }
    }

    fn get_tag(&self) -> u8 {
        0b11
    }

    pub fn get_encoding(&self) -> u8 {
        self.run | (self.get_tag() << 6)
    }
}

enum Chunk {
    RGB(OpRGB),
    RGBA(OpRGBA),
    Index(OpIndex),
    Diff(OpDiff),
    Luma(OpLuma),
    Run(OpRun),
}
