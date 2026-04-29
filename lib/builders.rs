use crate::{
    ByteAddress, ControllerAttachError, Instruction, IoController, Machine, Memory, SystemCall,
    SystemCallAttachError, WORD_SIZE_BYTES, image::Image,
};

pub fn recommended_stack_pointer() -> ByteAddress {
    ByteAddress(0xEFFF_FFFC)
}

pub fn recommended_stack_register() -> usize {
    crate::REGISTER_COUNT - 1
}

pub fn recommended_io_address() -> ByteAddress {
    ByteAddress(0xF000_0000)
}

/// A segment-oriented memory builder.
///
/// Segments are compile-time layout ranges. They constrain where sequential
/// writes may be placed, but they do not imply any runtime memory protection.
pub struct MemoryBuilder {
    mem: Memory,
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

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum MemoryBuilderError {
    #[error("invalid segment range: {start:#010x}..{end:#010x}")]
    InvalidSegmentRange {
        start: ByteAddress,
        end: ByteAddress,
    },
    #[error(
        "segment{} overlaps existing segment: {start:#010x}..{end:#010x}",
        format_segment_name(name)
    )]
    SegmentOverlap {
        name: Option<String>,
        start: ByteAddress,
        end: ByteAddress,
    },
    #[error("unknown segment")]
    UnknownSegment,
    #[error(
        "segment{} overflow: write of {len} bytes at {head:#010x} exceeds {end:#010x}",
        format_segment_name(name)
    )]
    SegmentOverflow {
        name: Option<String>,
        head: ByteAddress,
        len: u32,
        end: ByteAddress,
    },
    #[error("address overflow: advancing {head:#010x} by {len} bytes")]
    AddressOverflow { head: ByteAddress, len: u32 },
    #[error("write too large for memory builder: {len} bytes")]
    WriteTooLarge { len: usize },
}

impl MemoryBuilder {
    pub fn new() -> MemoryBuilder {
        MemoryBuilder {
            mem: Memory::new(),
            segments: Vec::new(),
        }
    }

    pub fn emit_to_image(self) -> Image {
        Image::from_memory(self.mem)
    }

    pub fn emit_to_machine(self) -> Machine {
        MachineBuilder::new().with_memory(self.mem).build()
    }

    pub fn clone_to_image(&self) -> Image {
        Image::from_memory(self.mem.snapshot())
    }

    pub fn clone_to_machine(&self) -> Machine {
        MachineBuilder::new()
            .with_memory(self.mem.snapshot())
            .build()
    }

    pub fn define_segment(
        &mut self,
        name: Option<&str>,
        start: ByteAddress,
        end: ByteAddress,
    ) -> Result<SegmentId, MemoryBuilderError> {
        if start >= end {
            return Err(MemoryBuilderError::InvalidSegmentRange { start, end });
        }

        for segment in &self.segments {
            if start < segment.end && segment.start < end {
                return Err(MemoryBuilderError::SegmentOverlap {
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

    pub fn segment_head(&self, id: SegmentId) -> Result<ByteAddress, MemoryBuilderError> {
        Ok(self.segment(id)?.head)
    }

    pub fn reserve_bytes(
        &mut self,
        id: SegmentId,
        len: u32,
    ) -> Result<ByteAddress, MemoryBuilderError> {
        self.advance_segment(id, len)
    }

    pub fn write_word(
        &mut self,
        id: SegmentId,
        data: u32,
    ) -> Result<ByteAddress, MemoryBuilderError> {
        let addr = self.advance_segment(id, 4)?;
        self.mem.write_raw_word(addr, data);
        Ok(addr)
    }

    pub fn write_half_word(
        &mut self,
        id: SegmentId,
        data: u16,
    ) -> Result<ByteAddress, MemoryBuilderError> {
        let addr = self.advance_segment(id, 2)?;
        self.mem.write_raw_half_word(addr, data);
        Ok(addr)
    }

    pub fn write_byte(
        &mut self,
        id: SegmentId,
        data: u8,
    ) -> Result<ByteAddress, MemoryBuilderError> {
        let addr = self.advance_segment(id, 1)?;
        self.mem.write_raw_byte(addr, data);
        Ok(addr)
    }

    pub fn write_bytes(
        &mut self,
        id: SegmentId,
        bytes: &[u8],
    ) -> Result<ByteAddress, MemoryBuilderError> {
        let len = u32::try_from(bytes.len())
            .map_err(|_| MemoryBuilderError::WriteTooLarge { len: bytes.len() })?;
        let addr = self.advance_segment(id, len)?;
        self.mem.write_raw_bytes(addr, bytes);
        Ok(addr)
    }

    fn segment(&self, id: SegmentId) -> Result<&Segment, MemoryBuilderError> {
        self.segments
            .get(id.0)
            .ok_or(MemoryBuilderError::UnknownSegment)
    }

    fn advance_segment(
        &mut self,
        id: SegmentId,
        len: u32,
    ) -> Result<ByteAddress, MemoryBuilderError> {
        let segment = self
            .segments
            .get_mut(id.0)
            .ok_or(MemoryBuilderError::UnknownSegment)?;
        let head = segment.head;
        let Some(next) = head.0.checked_add(len) else {
            return Err(MemoryBuilderError::AddressOverflow { head, len });
        };
        if next > segment.end.0 {
            return Err(MemoryBuilderError::SegmentOverflow {
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

pub struct MachineBuilder {
    machine: Machine,
}

impl MachineBuilder {
    pub fn new() -> MachineBuilder {
        let mut builder = MachineBuilder {
            machine: Machine::new(),
        };
        builder.use_recommended_stack();
        builder
    }

    #[cfg(test)]
    pub(crate) fn from_machine(machine: Machine) -> MachineBuilder {
        MachineBuilder { machine }
    }

    pub fn with_image(self, image: Image) -> MachineBuilder {
        self.with_memory(image.into_memory())
    }

    pub fn with_cloned_image(self, image: &Image) -> MachineBuilder {
        self.with_memory(image.clone_memory())
    }

    pub fn with_instructions(mut self, instructions: &[Instruction]) -> MachineBuilder {
        self.machine.override_with_instructions(instructions);
        self
    }

    pub fn attach_io_controller(
        &mut self,
        desired_address: Option<ByteAddress>,
        io: impl IoController + 'static,
    ) -> Result<ByteAddress, ControllerAttachError> {
        self.machine
            .attach_io_controller(desired_address, recommended_io_address(), io)
    }

    pub fn attach_system_call(
        &mut self,
        desired_call_number: u32,
        system_call: impl SystemCall + 'static,
    ) -> Result<u32, SystemCallAttachError> {
        self.machine
            .attach_system_call(desired_call_number, system_call)
    }

    pub fn build(self) -> Machine {
        self.machine
    }

    fn with_memory(mut self, mem: Memory) -> MachineBuilder {
        self.machine.mem = mem;
        self.machine.ios.clear();
        self
    }

    fn use_recommended_stack(&mut self) {
        debug_assert_eq!(recommended_stack_pointer().0 % WORD_SIZE_BYTES, 0);
        self.machine
            .write_register(recommended_stack_register(), recommended_stack_pointer().0);
    }
}

impl Default for MemoryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for MachineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        MachineBuilder, MemoryBuilder, recommended_stack_pointer, recommended_stack_register,
    };
    use crate::{Block, ByteAddress};

    #[test]
    fn memory_builder_segments_have_independent_heads() {
        let mut builder = MemoryBuilder::new();
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
    fn memory_builder_rejects_segment_overflow() {
        let mut builder = MemoryBuilder::new();
        let text = builder
            .define_segment(Some("text"), ByteAddress(0), ByteAddress(4))
            .unwrap();

        builder.write_word(text, 0).unwrap();
        let err = builder.write_byte(text, 1).unwrap_err().to_string();
        assert!(err.contains("segment `text` overflow"));
    }

    #[test]
    fn memory_builder_rejects_overlapping_segments() {
        let mut builder = MemoryBuilder::new();
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
    fn memory_builder_reserve_advances_without_emitting_bytes() {
        let mut builder = MemoryBuilder::new();
        let heap = builder
            .define_segment(Some("heap"), ByteAddress(0x2000), ByteAddress(0x3000))
            .unwrap();

        assert_eq!(
            builder.reserve_bytes(heap, 0x20).unwrap(),
            ByteAddress(0x2000)
        );
        assert_eq!(builder.segment_head(heap).unwrap(), ByteAddress(0x2020));

        let mut machine = builder.emit_to_machine();
        assert_eq!(machine.read_byte(ByteAddress(0x2000)), 0);
    }

    #[test]
    fn memory_builder_clones_without_consuming_builder() {
        let mut builder = MemoryBuilder::new();
        let data = builder
            .define_segment(Some("data"), ByteAddress(0x2000), ByteAddress(0x3000))
            .unwrap();
        builder.write_word(data, 0x1234_5678).unwrap();

        let mut image_machine = builder.clone_to_image().consume_to_machine();
        let mut cloned_machine = builder.clone_to_machine();
        let mut emitted_machine = builder.emit_to_machine();

        assert_eq!(image_machine.read_word(ByteAddress(0x2000)), 0x1234_5678);
        assert_eq!(cloned_machine.read_word(ByteAddress(0x2000)), 0x1234_5678);
        assert_eq!(emitted_machine.read_word(ByteAddress(0x2000)), 0x1234_5678);
    }

    #[test]
    fn machine_builder_loads_image_memory() {
        let mut memory = MemoryBuilder::new();
        let text = memory
            .define_segment(Some("text"), ByteAddress(0), ByteAddress(0x100))
            .unwrap();
        memory.write_word(text, 0x1234_5678).unwrap();

        let image = memory.emit_to_image();
        let mut machine = MachineBuilder::new().with_image(image).build();

        assert_eq!(machine.read_word(ByteAddress(0)), 0x1234_5678);
    }

    #[test]
    fn machine_builder_clones_image_memory() {
        let mut memory = MemoryBuilder::new();
        let text = memory
            .define_segment(Some("text"), ByteAddress(0), ByteAddress(0x100))
            .unwrap();
        memory.write_word(text, 0x1234_5678).unwrap();

        let image = memory.emit_to_image();
        let mut machine_a = MachineBuilder::new().with_cloned_image(&image).build();
        machine_a.write_word(ByteAddress(0), 0xAABB_CCDD);
        let mut machine_b = MachineBuilder::new().with_cloned_image(&image).build();

        assert_eq!(machine_a.read_word(ByteAddress(0)), 0xAABB_CCDD);
        assert_eq!(machine_b.read_word(ByteAddress(0)), 0x1234_5678);
    }

    #[test]
    fn machine_builder_loads_instructions() {
        let mut machine = MachineBuilder::new()
            .with_instructions(&[crate::is::addi(1, 0, 7), crate::Instruction::HALT])
            .build();

        machine.exec_while_not_halt().unwrap();

        assert_eq!(machine.read_register(1), 7);
    }

    #[test]
    fn machine_builder_uses_recommended_stack_pointer() {
        let machine = MachineBuilder::new().build();

        assert_eq!(
            machine.read_register(recommended_stack_register()),
            recommended_stack_pointer().0
        );
    }

    #[test]
    fn machine_builder_returns_machine_with_attached_io() {
        struct NoopController;

        impl crate::IoController for NoopController {
            fn read(
                &mut self,
                _mem: &mut crate::Memory,
                _addr: ByteAddress,
                _width: crate::Width,
            ) -> u32 {
                0
            }

            fn write(
                &mut self,
                _mem: &mut crate::Memory,
                _addr: ByteAddress,
                _width: crate::Width,
                _data: u32,
            ) {
            }
        }

        let mut builder = MachineBuilder::new();
        let addr = builder.attach_io_controller(None, NoopController).unwrap();
        let machine = builder.build();

        assert!(matches!(machine.mem.block_from_addr(addr).0, Block::Io));
    }
}
