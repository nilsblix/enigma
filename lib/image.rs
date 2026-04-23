use std::{
    error::Error,
    fmt,
    io::{self, Read, Write},
};

use super::{BLOCK_SIZE, Block, ByteAddress, ByteOffset, Instruction, Machine, Memory};

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

/// An opinionated builder of images. The builder makes assumptions concerning
/// the layout of the built machine's memory sections, ex: this struct contains
/// methods concerning text section and data section.
pub struct ImageBuilder {
    img: Image,
    text_head: ByteAddress,
    data_head: ByteAddress,

    data_start_addr: ByteAddress,
    /// The maximum amount of bytes stored in the data section.
    data_max_size: u32,
}

#[derive(Debug)]
pub enum BuilderError {
    TextOverflow,
    DataOverflow,
}

impl fmt::Display for BuilderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BuilderError::TextOverflow => write!(f, "text-section overflow"),
            BuilderError::DataOverflow => write!(f, "data-section overflow"),
        }
    }
}

impl Error for BuilderError {}

impl ImageBuilder {
    pub fn new() -> ImageBuilder {
        let data_start_addr = ByteAddress(0x10000000);
        ImageBuilder {
            img: Image::new(),
            text_head: ByteAddress::ZERO,
            data_head: data_start_addr,
            data_start_addr,
            data_max_size: 0x10000000,
        }
    }

    pub fn emit_image(self) -> Image {
        self.img
    }

    pub fn write_text_word(&mut self, text: u32) -> Result<ByteAddress, BuilderError> {
        if self.text_head.0 > self.data_start_addr.0 - 4 {
            return Err(BuilderError::TextOverflow);
        }
        self.img.write_word(self.text_head, text);
        let re = self.text_head;
        self.text_head = self.text_head.overflowing_add_bytes(ByteOffset(4)).0;
        Ok(re)
    }

    pub fn write_text_half_word(&mut self, text: u16) -> Result<ByteAddress, BuilderError> {
        if self.text_head.0 > self.data_start_addr.0 - 2 {
            return Err(BuilderError::TextOverflow);
        }
        self.img.write_half_word(self.text_head, text);
        let re = self.text_head;
        self.text_head = self.text_head.overflowing_add_bytes(ByteOffset(2)).0;
        Ok(re)
    }

    pub fn write_text_byte(&mut self, text: u8) -> Result<ByteAddress, BuilderError> {
        if self.text_head.0 > self.data_start_addr.0 - 1 {
            return Err(BuilderError::TextOverflow);
        }
        self.img.write_byte(self.text_head, text);
        let re = self.text_head;
        self.text_head = self.text_head.overflowing_add_bytes(ByteOffset(1)).0;
        Ok(re)
    }

    pub fn write_text_bytes(&mut self, text: &[u8]) -> Result<ByteAddress, BuilderError> {
        let len = text.len() as u32;
        if self.text_head.0 > self.data_start_addr.0 - len {
            return Err(BuilderError::TextOverflow);
        }
        self.img.write_bytes(self.text_head, text);
        let re = self.text_head;
        self.text_head = self
            .text_head
            .overflowing_add_bytes(ByteOffset(len as i32))
            .0;
        Ok(re)
    }

    pub fn write_data_word(&mut self, data: u32) -> Result<ByteAddress, BuilderError> {
        if self.data_head.0 > self.data_start_addr.0 + self.data_max_size - 4 {
            return Err(BuilderError::DataOverflow);
        }
        self.img.write_word(self.data_head, data);
        let re = self.data_head;
        self.data_head = self.data_head.overflowing_add_bytes(ByteOffset(4)).0;
        Ok(re)
    }

    pub fn write_data_half_word(&mut self, data: u16) -> Result<ByteAddress, BuilderError> {
        if self.data_head.0 > self.data_start_addr.0 + self.data_max_size - 2 {
            return Err(BuilderError::DataOverflow);
        }
        self.img.write_half_word(self.data_head, data);
        let re = self.data_head;
        self.data_head = self.data_head.overflowing_add_bytes(ByteOffset(2)).0;
        Ok(re)
    }

    pub fn write_data_byte(&mut self, data: u8) -> Result<ByteAddress, BuilderError> {
        if self.data_head.0 > self.data_start_addr.0 + self.data_max_size - 1 {
            return Err(BuilderError::DataOverflow);
        }
        self.img.write_byte(self.data_head, data);
        let re = self.data_head;
        self.data_head = self.data_head.overflowing_add_bytes(ByteOffset(1)).0;
        Ok(re)
    }

    pub fn write_data_bytes(&mut self, data: &[u8]) -> Result<ByteAddress, BuilderError> {
        let len = data.len() as u32;
        if self.data_head.0 > self.data_start_addr.0 + self.data_max_size - len {
            return Err(BuilderError::DataOverflow);
        }
        self.img.write_bytes(self.data_head, data);
        let re = self.data_head;
        self.data_head = self
            .data_head
            .overflowing_add_bytes(ByteOffset(len as i32))
            .0;
        Ok(re)
    }
}
