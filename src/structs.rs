#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Channels {
    RGB = 3,
    RGBA = 4,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColorSpace {
    SRGB = 0,
    Linear = 1,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QoiHeader {
    magic: [u8; 4],
    pub width: u32,
    pub height: u32,
    pub channels: Channels,
    pub color_space: ColorSpace,
}

impl QoiHeader {
    pub fn from_u8(header: &[u8]) -> Result<Self, String> {
        if header.len() < 14 {
            return Err(format!("Header is too short: {}", header.len()));
        }
        if header[0] != "qoif".as_bytes()[0]
            || header[1] != "qoif".as_bytes()[1]
            || header[2] != "qoif".as_bytes()[2]
            || header[3] != "qoif".as_bytes()[3]
        {
            return Err(format!("Invalid magic: {:?}", &header[0..4]));
        }

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
            _ => return Err(format!("Invalid number of channels: {}", header[12])),
        };
        let colorspace = match header[13] {
            0 => ColorSpace::SRGB,
            1 => ColorSpace::Linear,
            _ => return Err(format!("Invalid color space: {}", header[13])),
        };
        Ok(QoiHeader {
            magic: [header[0], header[1], header[2], header[3]],
            width,
            height,
            channels,
            color_space: colorspace,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pixel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
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

impl Pixel {
    pub fn random() -> Self {
        Pixel {
            r: rand::random(),
            g: rand::random(),
            b: rand::random(),
            a: 255,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct QOIHash {
    data: Box<[Pixel]>,
}

impl QOIHash {
    pub(crate) fn new() -> Self {
        QOIHash {
            data: vec![Pixel::default(); 64].into_boxed_slice(),
        }
    }

    pub(crate) fn get_index(&self, pixel: &Pixel) -> usize {
        ((pixel
            .r
            .wrapping_mul(3)
            .wrapping_add(pixel.g.wrapping_mul(5))
            .wrapping_add(pixel.b.wrapping_mul(7))
            .wrapping_add(pixel.a.wrapping_mul(11)))
            % 64) as usize
    }

    pub(crate) fn get(&self, index: u8) -> Pixel {
        self.data[index as usize]
    }

    pub(crate) fn lookup(&mut self, pixel: &Pixel) -> Option<u8> {
        let index = self.get_index(pixel);
        if self.data[index] == *pixel {
            return Some(index as u8);
        }
        return None;
    }

    pub(crate) fn insert(&mut self, pixel: &Pixel) {
        let pos = self.get_index(pixel);
        self.data[pos] = *pixel;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OpRGB {
    pub(crate) r: u8,
    pub(crate) g: u8,
    pub(crate) b: u8,
}

impl OpRGB {
    pub(crate) fn new(r: u8, g: u8, b: u8) -> Self {
        OpRGB { r, g, b }
    }

    fn get_tag(&self) -> u8 {
        0b11111110
    }

    pub fn get_encoding(&self) -> [u8; 4] {
        [self.get_tag(), self.r, self.g, self.b]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OpRGBA {
    pub(crate) r: u8,
    pub(crate) g: u8,
    pub(crate) b: u8,
    pub(crate) a: u8,
}

impl OpRGBA {
    pub(crate) fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        OpRGBA { r, g, b, a }
    }

    fn get_tag(&self) -> u8 {
        0b11111111
    }

    pub fn get_encoding(&self) -> [u8; 5] {
        [self.get_tag(), self.r, self.g, self.b, self.a]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OpIndex {
    pub(crate) index: u8,
}

impl OpIndex {
    pub(crate) fn new(index: u8) -> Self {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OpDiff {
    pub(crate) diff: u8,
}

impl OpDiff {
    pub(crate) fn new(r: i8, g: i8, b: i8) -> Self {
        debug_assert!(r >= -2 && r <= 1);
        debug_assert!(g >= -2 && g <= 1);
        debug_assert!(b >= -2 && b <= 1);
        OpDiff {
            diff: (((r + 2) as u8) << 4) | (((g + 2) as u8) << 2) | ((b + 2) as u8),
        }
    }

    fn get_tag(&self) -> u8 {
        0b01
    }

    pub fn get_encoding(&self) -> u8 {
        self.diff | (self.get_tag() << 6)
    }

    pub fn get_diffs(&self) -> (i8, i8, i8) {
        let r = ((self.diff >> 4) & 0b11) as i8 - 2;
        let g = ((self.diff >> 2) & 0b11) as i8 - 2;
        let b = ((self.diff >> 0) & 0b11) as i8 - 2;
        (r, g, b)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OpLuma {
    pub(crate) dg: u8,
    pub(crate) rb: u8,
}

impl OpLuma {
    pub(crate) fn new(dr: i8, dg: i8, db: i8) -> Self {
        debug_assert!(dg >= -32 && dg <= 31);
        debug_assert!(dr - dg >= -8 && dr - dg <= 7);
        debug_assert!(db - dg >= -8 && db - dg <= 7);
        OpLuma {
            dg: (dg + 32) as u8,
            rb: ((dr - dg + 8) as u8) << 4 | ((db - dg + 8) as u8),
        }
    }

    fn get_tag(&self) -> u8 {
        0b10
    }

    pub fn get_encoding(&self) -> [u8; 2] {
        [self.get_tag() << 6 | self.dg, self.rb]
    }

    pub fn get_diffs(&self) -> (i8, i8, i8) {
        let dg = self.dg as i8 - 32;
        let dr = ((self.rb >> 4) & 0b1111) as i8 - 8;
        let db = ((self.rb >> 0) & 0b1111) as i8 - 8;
        (dr + dg, dg, db + dg)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OpRun {
    pub(crate) run: u8,
}

impl OpRun {
    pub(crate) fn new(run: u8) -> Self {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Chunk {
    RGB(OpRGB),
    RGBA(OpRGBA),
    Index(OpIndex),
    Diff(OpDiff),
    Luma(OpLuma),
    Run(OpRun),
}

impl Chunk {
    pub(crate) fn from_encoding(possible_chunk: &[u8]) -> Self {
        // dbg!(&possible_chunk);
        let op = possible_chunk[0];
        match (op & 0b11000000) >> 6 {
            0b00 => Chunk::Index(OpIndex::new(op & 0b00111111)),
            0b01 => Chunk::Diff(OpDiff {
                diff: op & 0b00111111,
            }),
            0b10 => Chunk::Luma(OpLuma {
                dg: op & 0b00111111,
                rb: possible_chunk[1],
            }),
            0b11 => {
                if op == 0b11111110 {
                    Chunk::RGB(OpRGB::new(
                        possible_chunk[1],
                        possible_chunk[2],
                        possible_chunk[3],
                    ))
                } else if op == 0b11111111 {
                    Chunk::RGBA(OpRGBA::new(
                        possible_chunk[1],
                        possible_chunk[2],
                        possible_chunk[3],
                        possible_chunk[4],
                    ))
                } else {
                    Chunk::Run(OpRun::new(op & 0b00111111))
                }
            }
            _ => unreachable!(),
        }
    }
}
