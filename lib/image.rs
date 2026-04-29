use std::io::{self, Read, Write};

use super::{BLOCK_SIZE, Block, ByteAddress, Instruction, Machine, Memory};

pub struct Image {
    mem: Memory,
}

const IMAGE_MAGIC: [u8; 4] = *b"EVM1";

impl Image {
    pub fn new() -> Image {
        Image { mem: Memory::new() }
    }

    pub(crate) fn from_memory(mem: Memory) -> Image {
        Image { mem }
    }

    pub(crate) fn into_memory(self) -> Memory {
        self.mem
    }

    pub(crate) fn clone_memory(&self) -> Memory {
        self.mem.snapshot()
    }

    pub fn from_chunk_bytes(bytes: &[u8]) -> Result<Image, ImageError> {
        if bytes.len() < IMAGE_MAGIC.len() || bytes[..IMAGE_MAGIC.len()] != IMAGE_MAGIC {
            return Err(ImageError::InvalidHeader);
        }

        let mut image = Image::new();
        let mut cursor = IMAGE_MAGIC.len();

        while cursor < bytes.len() {
            if bytes.len() - cursor < 8 {
                return Err(ImageError::TruncatedChunkHeader);
            }

            let addr = u32::from_be_bytes(bytes[cursor..cursor + 4].try_into().unwrap());
            cursor += 4;
            let len = u32::from_be_bytes(bytes[cursor..cursor + 4].try_into().unwrap());
            cursor += 4;

            let len = len as usize;
            if bytes.len() - cursor < len {
                return Err(ImageError::TruncatedChunkPayload);
            }

            let chunk = &bytes[cursor..cursor + len];
            cursor += len;
            image.write_bytes(ByteAddress(addr), chunk);
        }

        Ok(image)
    }

    pub fn load_chunks(reader: &mut impl Read) -> Result<Image, ImageError> {
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

#[derive(Debug, thiserror::Error)]
pub enum ImageError {
    #[error("invalid EVM image header")]
    InvalidHeader,
    #[error("truncated EVM chunk header")]
    TruncatedChunkHeader,
    #[error("truncated EVM chunk payload")]
    TruncatedChunkPayload,
    #[error("could not read EVM image chunks: {0}")]
    Read(#[from] io::Error),
}

#[cfg(test)]
mod tests {
    use super::{Image, ImageError};

    #[test]
    fn image_rejects_invalid_header() {
        let err = match Image::from_chunk_bytes(b"nope") {
            Ok(_) => panic!("image decode should fail"),
            Err(err) => err,
        };
        assert!(matches!(err, ImageError::InvalidHeader));
    }

    #[test]
    fn image_rejects_truncated_chunk_header() {
        let err = match Image::from_chunk_bytes(b"EVM1\0") {
            Ok(_) => panic!("image decode should fail"),
            Err(err) => err,
        };
        assert!(matches!(err, ImageError::TruncatedChunkHeader));
    }

    #[test]
    fn image_rejects_truncated_chunk_payload() {
        let bytes = [b'E', b'V', b'M', b'1', 0, 0, 0, 0, 0, 0, 0, 4, 0xaa, 0xbb];
        let err = match Image::from_chunk_bytes(&bytes) {
            Ok(_) => panic!("image decode should fail"),
            Err(err) => err,
        };
        assert!(matches!(err, ImageError::TruncatedChunkPayload));
    }
}
