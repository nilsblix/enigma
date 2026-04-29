use std::{
    error::Error,
    fmt,
    io::{self, Read, Write},
};

use super::{BLOCK_SIZE, Block, ByteAddress, Instruction, Machine, Memory};

pub struct Image {
    mem: Memory,
}

const IMAGE_MAGIC: [u8; 4] = *b"EVM1";

impl Image {
    pub fn new() -> Image {
        Image { mem: Memory::new() }
    }

    pub fn from_chunk_bytes(bytes: &[u8]) -> io::Result<Image> {
        if bytes.len() < IMAGE_MAGIC.len() || bytes[..IMAGE_MAGIC.len()] != IMAGE_MAGIC {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid EVM image header",
            ));
        }

        let mut image = Image::new();
        let mut cursor = IMAGE_MAGIC.len();

        while cursor < bytes.len() {
            if bytes.len() - cursor < 8 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "truncated EVM chunk header",
                ));
            }

            let addr = u32::from_be_bytes(bytes[cursor..cursor + 4].try_into().unwrap());
            cursor += 4;
            let len = u32::from_be_bytes(bytes[cursor..cursor + 4].try_into().unwrap());
            cursor += 4;

            let len = len as usize;
            if bytes.len() - cursor < len {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "truncated EVM chunk payload",
                ));
            }

            let chunk = &bytes[cursor..cursor + len];
            cursor += len;
            image.write_bytes(ByteAddress(addr), chunk);
        }

        Ok(image)
    }

    pub fn load_chunks(reader: &mut impl Read) -> io::Result<Image> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes)?;
        Image::from_chunk_bytes(bytes.as_slice())
    }

    pub fn dump_chunks(&self, writer: &mut impl Write) -> io::Result<()> {
        writer.write_all(&IMAGE_MAGIC)?;

        for (block_index, block) in self.mem.blocks.iter().enumerate() {
            let mem = match block {
                Block::Memory(mem) => mem,
                Block::Empty | Block::Io => continue,
            };
            let mut cursor = 0usize;
            while cursor < BLOCK_SIZE {
                let start = match mem[cursor..].iter().position(|&byte| byte != 0) {
                    Some(offset) => cursor + offset,
                    None => break,
                };
                let end = match mem[start..].iter().position(|&byte| byte == 0) {
                    Some(offset) => start + offset,
                    None => BLOCK_SIZE,
                };
                let addr = ((block_index * BLOCK_SIZE) + start) as u32;
                let bytes = &mem[start..end];

                writer.write_all(&addr.to_be_bytes())?;
                writer.write_all(&(bytes.len() as u32).to_be_bytes())?;
                writer.write_all(bytes)?;
                cursor = end;
            }
        }

        Ok(())
    }

    pub fn branch_to_machine(&self) -> Machine {
        let mut m = Machine::new();
        m.mem = self.mem.snapshot();
        m
    }

    pub fn consume_to_machine(self) -> Machine {
        let mut m = Machine::new();
        m.mem = self.mem;
        m
    }

    pub fn write_byte(&mut self, addr: ByteAddress, data: u8) {
        self.mem.write_raw_byte(addr, data);
    }

    pub fn write_half_word(&mut self, addr: ByteAddress, data: u16) {
        self.mem.write_raw_half_word(addr, data);
    }

    pub fn write_word(&mut self, addr: ByteAddress, data: u32) {
        self.mem.write_raw_word(addr, data);
    }

    pub fn write_bytes(&mut self, addr: ByteAddress, bytes: &[u8]) {
        self.mem.write_raw_bytes(addr, bytes);
    }

    pub fn override_with_instructions(&mut self, instructions: &[Instruction]) {
        let mut addr = ByteAddress::ZERO;
        for instruction in instructions {
            self.write_word(addr, instruction.encode());
            addr = addr.next_word().0;
        }
    }
}

/// A segment-oriented image builder.
///
/// Segments are compile-time layout ranges. They constrain where sequential
/// writes may be placed, but they do not imply any runtime memory protection.
pub struct ImageBuilder {
    img: Image,
    segments: Vec<Segment>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SegmentId(usize);

struct Segment {
    name: Option<String>,
    start: ByteAddress,
    end: ByteAddress,
    head: ByteAddress,
}

#[derive(Debug)]
pub enum BuilderError {
    InvalidSegmentRange {
        start: ByteAddress,
        end: ByteAddress,
    },
    SegmentOverlap {
        name: Option<String>,
        start: ByteAddress,
        end: ByteAddress,
    },
    UnknownSegment,
    SegmentOverflow {
        name: Option<String>,
        head: ByteAddress,
        len: u32,
        end: ByteAddress,
    },
    AddressOverflow {
        head: ByteAddress,
        len: u32,
    },
    WriteTooLarge {
        len: usize,
    },
}

impl fmt::Display for BuilderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BuilderError::InvalidSegmentRange { start, end } => {
                write!(
                    f,
                    "invalid segment range: {:#010x}..{:#010x}",
                    start.0, end.0
                )
            }
            BuilderError::SegmentOverlap { name, start, end } => {
                write!(
                    f,
                    "segment{} overlaps existing segment: {:#010x}..{:#010x}",
                    format_segment_name(name),
                    start.0,
                    end.0
                )
            }
            BuilderError::UnknownSegment => write!(f, "unknown segment"),
            BuilderError::SegmentOverflow {
                name,
                head,
                len,
                end,
            } => {
                write!(
                    f,
                    "segment{} overflow: write of {} bytes at {:#010x} exceeds {:#010x}",
                    format_segment_name(name),
                    len,
                    head.0,
                    end.0
                )
            }
            BuilderError::AddressOverflow { head, len } => {
                write!(
                    f,
                    "address overflow: advancing {:#010x} by {} bytes",
                    head.0, len
                )
            }
            BuilderError::WriteTooLarge { len } => {
                write!(f, "write too large for image builder: {len} bytes")
            }
        }
    }
}

impl Error for BuilderError {}

impl ImageBuilder {
    pub fn new() -> ImageBuilder {
        ImageBuilder {
            img: Image::new(),
            segments: Vec::new(),
        }
    }

    pub fn emit_image(self) -> Image {
        self.img
    }

    pub fn define_segment(
        &mut self,
        name: Option<&str>,
        start: ByteAddress,
        end: ByteAddress,
    ) -> Result<SegmentId, BuilderError> {
        if start >= end {
            return Err(BuilderError::InvalidSegmentRange { start, end });
        }

        for segment in &self.segments {
            if start < segment.end && segment.start < end {
                return Err(BuilderError::SegmentOverlap {
                    name: name.map(str::to_owned),
                    start,
                    end,
                });
            }
        }

        let id = SegmentId(self.segments.len());
        self.segments.push(Segment {
            name: name.map(str::to_owned),
            start,
            end,
            head: start,
        });
        Ok(id)
    }

    pub fn segment_head(&self, id: SegmentId) -> Result<ByteAddress, BuilderError> {
        Ok(self.segment(id)?.head)
    }

    pub fn reserve_bytes(&mut self, id: SegmentId, len: u32) -> Result<ByteAddress, BuilderError> {
        self.advance_segment(id, len)
    }

    pub fn write_word(&mut self, id: SegmentId, data: u32) -> Result<ByteAddress, BuilderError> {
        let addr = self.advance_segment(id, 4)?;
        self.img.write_word(addr, data);
        Ok(addr)
    }

    pub fn write_half_word(
        &mut self,
        id: SegmentId,
        data: u16,
    ) -> Result<ByteAddress, BuilderError> {
        let addr = self.advance_segment(id, 2)?;
        self.img.write_half_word(addr, data);
        Ok(addr)
    }

    pub fn write_byte(&mut self, id: SegmentId, data: u8) -> Result<ByteAddress, BuilderError> {
        let addr = self.advance_segment(id, 1)?;
        self.img.write_byte(addr, data);
        Ok(addr)
    }

    pub fn write_bytes(
        &mut self,
        id: SegmentId,
        bytes: &[u8],
    ) -> Result<ByteAddress, BuilderError> {
        let len = u32::try_from(bytes.len())
            .map_err(|_| BuilderError::WriteTooLarge { len: bytes.len() })?;
        let addr = self.advance_segment(id, len)?;
        self.img.write_bytes(addr, bytes);
        Ok(addr)
    }

    fn segment(&self, id: SegmentId) -> Result<&Segment, BuilderError> {
        self.segments.get(id.0).ok_or(BuilderError::UnknownSegment)
    }

    fn advance_segment(&mut self, id: SegmentId, len: u32) -> Result<ByteAddress, BuilderError> {
        let segment = self
            .segments
            .get_mut(id.0)
            .ok_or(BuilderError::UnknownSegment)?;
        let head = segment.head;
        let Some(next) = head.0.checked_add(len) else {
            return Err(BuilderError::AddressOverflow { head, len });
        };
        if next > segment.end.0 {
            return Err(BuilderError::SegmentOverflow {
                name: segment.name.clone(),
                head,
                len,
                end: segment.end,
            });
        }
        segment.head = ByteAddress(next);
        Ok(head)
    }
}

fn format_segment_name(name: &Option<String>) -> String {
    match name {
        Some(name) => format!(" `{name}`"),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::{Image, ImageBuilder};
    use crate::ByteAddress;

    #[test]
    fn builder_segments_have_independent_heads() {
        let mut builder = ImageBuilder::new();
        let text = builder
            .define_segment(Some("text"), ByteAddress(0), ByteAddress(0x100))
            .unwrap();
        let data = builder
            .define_segment(Some("data"), ByteAddress(0x1000), ByteAddress(0x1100))
            .unwrap();

        assert_eq!(
            builder.write_word(text, 0x1122_3344).unwrap(),
            ByteAddress(0)
        );
        assert_eq!(
            builder.write_bytes(data, b"abc").unwrap(),
            ByteAddress(0x1000)
        );
        assert_eq!(builder.segment_head(text).unwrap(), ByteAddress(4));
        assert_eq!(builder.segment_head(data).unwrap(), ByteAddress(0x1003));
    }

    #[test]
    fn builder_rejects_segment_overflow() {
        let mut builder = ImageBuilder::new();
        let text = builder
            .define_segment(Some("text"), ByteAddress(0), ByteAddress(4))
            .unwrap();

        builder.write_word(text, 0).unwrap();
        let err = builder.write_byte(text, 1).unwrap_err().to_string();
        assert!(err.contains("segment `text` overflow"));
    }

    #[test]
    fn builder_rejects_overlapping_segments() {
        let mut builder = ImageBuilder::new();
        builder
            .define_segment(Some("a"), ByteAddress(0), ByteAddress(0x100))
            .unwrap();
        let err = builder
            .define_segment(Some("b"), ByteAddress(0x80), ByteAddress(0x180))
            .unwrap_err()
            .to_string();

        assert!(err.contains("segment `b` overlaps existing segment"));
    }

    #[test]
    fn builder_reserve_advances_without_emitting_bytes() {
        let mut builder = ImageBuilder::new();
        let heap = builder
            .define_segment(Some("heap"), ByteAddress(0x2000), ByteAddress(0x3000))
            .unwrap();

        assert_eq!(
            builder.reserve_bytes(heap, 0x20).unwrap(),
            ByteAddress(0x2000)
        );
        assert_eq!(builder.segment_head(heap).unwrap(), ByteAddress(0x2020));

        let image: Image = builder.emit_image();
        let mut machine = image.consume_to_machine();
        assert_eq!(machine.read_byte(ByteAddress(0x2000)), 0);
    }
}
