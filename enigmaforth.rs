use enigma::{Builder, ByteAddress, ByteOffset, WordOffset, is};
use is::Instruction as I;

struct Putter {
    builder: Builder,
    head: ByteAddress,
    // TODO: Move this into some other structure.
    latest: ByteAddress,
}

impl Putter {
    const DICT_START: ByteAddress = ByteAddress(4);

    fn new() -> Putter {
        Putter {
            builder: Builder::new(),
            head: Self::DICT_START,
            latest: ByteAddress::ZERO,
        }
    }

    /// `amount_of_bytes` is not a ByteOffset due to `head` only moving forwards
    /// in the address space.
    ///
    /// We also ignore overflow.
    fn increment_head(&mut self, amount_of_bytes: u32) {
        self.head = self
            .head
            .overflowing_add_bytes(ByteOffset(amount_of_bytes as i32))
            .0;
    }

    fn byte(&mut self, b: u8) {
        self.builder.write_byte(self.head, b);
        self.increment_head(1);
    }

    fn bytes(&mut self, bytes: &[u8]) {
        self.builder.write_bytes(self.head, bytes);
        self.increment_head(bytes.len() as u32);
    }

    fn align_by_word(&mut self) {
        let pad = (4 - (self.head.0 % 4)) % 4;
        self.increment_head(pad);
    }

    fn word(&mut self, word: u32) {
        self.builder.write_word(self.head, word);
        self.increment_head(4);
    }

    fn inst(&mut self, i: is::Instruction) {
        self.word(i.encode());
    }
}

mod asm {
    use super::*;

    pub fn next(p: &mut Putter) {
        p.inst(I::ldw(10, 11, 0));
        p.inst(I::addi(11, 11, 4));
        p.inst(I::ldw(12, 10, 0));
        p.inst(I::jmpr(0, 12, 0));
    }

    pub fn push_return_stack(p: &mut Putter, reg: usize) {
        p.inst(I::stw(reg, 30, 0));
        p.inst(I::subi(30, 30, 4));
    }

    pub fn pop_return_stack(p: &mut Putter, reg: usize) {
        p.inst(I::addi(30, 30, 4));
        p.inst(I::ldw(reg, 30, 0));
    }

    pub fn do_colon(p: &mut Putter) {
        push_return_stack(p, 11);
        p.inst(I::addi(11, 10, 4));
        next(p)
    }

    pub fn exit(p: &mut Putter) {
        pop_return_stack(p, 11);
        next(p);
    }

    pub fn push_param_stack(p: &mut Putter, reg: usize) {
        p.inst(I::stw(reg, 31, 0));
        p.inst(I::subi(31, 31, 4));
    }

    pub fn pop_param_stack(p: &mut Putter, reg: usize) {
        p.inst(I::addi(31, 31, 4));
        p.inst(I::ldw(reg, 31, 0));
    }
}

const F_IMM_SHIFT: u8 = 7;
const F_HID_SHIFT: u8 = 5;
const F_NAME_MASK: u8 = 0b00011111;

/// Layout of the flag/name byte:
/// ```text
/// 0b10101010
///   ^ ^^
///   | |and onwards is the length, stored as u5.
///   | |
///   | F_HIDDEN
///   |
///   F_IMMED
/// ```
///
/// The middle bit of flag currently means nothing.
fn define_until_code_or_panic<'p, 'w>(p: &'p mut Putter, flags: (bool, bool), name: &'w [u8]) {
    if name.len() >= 32 {
        panic!("tried to define a word with a name longer than 32 bytes");
    }

    let word_addr = p.head;
    p.word(p.latest.0);

    let len = name.len() as u8;
    let mut flag_len = 0u8;
    flag_len = flag_len | ((flags.0 as u8) << F_IMM_SHIFT);
    flag_len = flag_len | ((flags.1 as u8) << F_HID_SHIFT);
    flag_len = flag_len | (len & F_NAME_MASK);
    p.byte(flag_len);
    p.bytes(name);
    p.align_by_word();
    p.latest = word_addr;
}

fn define_builtin_or_panic<'p, 'w, F>(
    p: &'p mut Putter,
    flags: (bool, bool),
    name: &'w [u8],
    body: F,
) where
    F: FnOnce(&mut Putter),
{
    define_until_code_or_panic(p, flags, name);
    // Codeword points to the body in a builtin, due to the builtin's
    // implementation being written in pure assembly.
    let body_addr = p.head.0 + 4;
    p.word(body_addr);
    body(p);
}

fn define_variable_or_panic<'p, 'w>(
    p: &'p mut Putter,
    flags: (bool, bool),
    name: &'w [u8],
    default_value: Option<u32>,
) -> ByteAddress {
    define_builtin_or_panic(p, flags, name, |p| {
        // Backpatch the variable cell address once the code body has been
        // fully emitted.
        let lo_patch_addr = p.head;
        p.inst(I::xori(15, 0, 0));
        let hi_patch_addr = p.head;
        p.inst(I::orui(15, 15, 0));
        asm::push_param_stack(p, 15);
        asm::next(p);

        let ptr = p.head;
        p.builder
            .write_word(lo_patch_addr, I::xori(15, 0, ptr.0 as u16).encode());
        p.builder.write_word(
            hi_patch_addr,
            I::orui(15, 15, (ptr.0 >> 16) as u16).encode(),
        );
        p.word(default_value.unwrap_or(0));
    });

    p.head.overflowing_add_words(WordOffset(-1)).0
}

fn define_builtin_words(p: &mut Putter) {
    ////////////////////////////////////////////////////////////////////////////
    // Common words
    ////////////////////////////////////////////////////////////////////////////

    define_builtin_or_panic(p, (false, false), "drop".as_bytes(), |p| {
        asm::pop_param_stack(p, 0);
        asm::next(p);
    });

    define_builtin_or_panic(p, (false, false), "swap".as_bytes(), |p| {
        asm::pop_param_stack(p, 15);
        asm::pop_param_stack(p, 16);
        asm::push_param_stack(p, 15);
        asm::push_param_stack(p, 16);
        asm::next(p);
    });

    define_builtin_or_panic(p, (false, false), "dup".as_bytes(), |p| {
        p.inst(I::ldw(15, 31, 4));
        asm::push_param_stack(p, 15);
        asm::next(p);
    });

    define_builtin_or_panic(p, (false, false), "over".as_bytes(), |p| {
        p.inst(I::ldw(15, 31, 8));
        asm::push_param_stack(p, 15);
        asm::next(p);
    });

    define_builtin_or_panic(p, (false, false), "rot".as_bytes(), |p| {
        // c b a -- b a c
        asm::pop_param_stack(p, 15);
        asm::pop_param_stack(p, 16);
        asm::pop_param_stack(p, 17);
        asm::push_param_stack(p, 16);
        asm::push_param_stack(p, 15);
        asm::push_param_stack(p, 17);
        asm::next(p);
    });

    define_builtin_or_panic(p, (false, false), "-rot".as_bytes(), |p| {
        // c b a -- a c b
        asm::pop_param_stack(p, 15);
        asm::pop_param_stack(p, 16);
        asm::pop_param_stack(p, 17);
        asm::push_param_stack(p, 15);
        asm::push_param_stack(p, 17);
        asm::push_param_stack(p, 16);
        asm::next(p);
    });

    define_builtin_or_panic(p, (false, false), "2drop".as_bytes(), |p| {
        // a b --
        asm::pop_param_stack(p, 0);
        asm::pop_param_stack(p, 0);
        asm::next(p);
    });

    define_builtin_or_panic(p, (false, false), "2dup".as_bytes(), |p| {
        // a b -- a b a b
        p.inst(I::ldw(15, 31, 8));
        p.inst(I::ldw(16, 31, 4));
        asm::push_param_stack(p, 15);
        asm::push_param_stack(p, 16);
        asm::next(p);
    });

    define_builtin_or_panic(p, (false, false), "2swap".as_bytes(), |p| {
        // a b c d -- c d a b
        asm::pop_param_stack(p, 15);
        asm::pop_param_stack(p, 16);
        asm::pop_param_stack(p, 17);
        asm::pop_param_stack(p, 18);
        asm::push_param_stack(p, 16);
        asm::push_param_stack(p, 15);
        asm::push_param_stack(p, 18);
        asm::push_param_stack(p, 17);
        asm::next(p);
    });

    define_builtin_or_panic(p, (false, false), "?dup".as_bytes(), |p| {
        // duplicate the top if non-zero.
        p.inst(I::ldw(15, 31, 4));
        p.inst(I::beq(15, 0, 3));
        asm::push_param_stack(p, 15); // this is really two instructions,
        // therefore we branch by 3.
        asm::next(p);
    });

    ////////////////////////////////////////////////////////////////////////////
    // Arithmetic
    ////////////////////////////////////////////////////////////////////////////

    define_builtin_or_panic(p, (false, false), "+".as_bytes(), |p| {
        asm::pop_param_stack(p, 15);
        asm::pop_param_stack(p, 16);
        p.inst(I::add(15, 15, 16));
        asm::push_param_stack(p, 15);
        asm::next(p);
    });

    define_builtin_or_panic(p, (false, false), "-".as_bytes(), |p| {
        asm::pop_param_stack(p, 15);
        asm::pop_param_stack(p, 16);
        p.inst(I::sub(16, 16, 15));
        asm::push_param_stack(p, 16);
        asm::next(p);
    });

    define_builtin_or_panic(p, (false, false), "=".as_bytes(), |p| {
        // top two are equal?
        asm::pop_param_stack(p, 15);
        asm::pop_param_stack(p, 16);
        p.inst(I::eql(15, 15, 16));
        asm::push_param_stack(p, 15);
        asm::next(p);
    });

    define_builtin_or_panic(p, (false, false), "<>".as_bytes(), |p| {
        // top two are not-equal?
        asm::pop_param_stack(p, 15);
        asm::pop_param_stack(p, 16);
        p.inst(I::eql(15, 15, 16));
        // bitwise negates r15.
        p.inst(I::xori(15, 15, 1));
        asm::push_param_stack(p, 15);
        asm::next(p);
    });

    define_builtin_or_panic(p, (false, false), "<".as_bytes(), |p| {
        asm::pop_param_stack(p, 15);
        asm::pop_param_stack(p, 16);
        p.inst(I::slt(15, 16, 15));
        asm::push_param_stack(p, 15);
        asm::next(p);
    });

    define_builtin_or_panic(p, (false, false), "<=".as_bytes(), |p| {
        asm::pop_param_stack(p, 15);
        asm::pop_param_stack(p, 16);
        p.inst(I::slt(17, 16, 15));
        p.inst(I::eql(18, 16, 15));
        p.inst(I::or(15, 17, 18));
        asm::push_param_stack(p, 15);
        asm::next(p);
    });

    define_builtin_or_panic(p, (false, false), ">".as_bytes(), |p| {
        asm::pop_param_stack(p, 15);
        asm::pop_param_stack(p, 16);
        p.inst(I::slt(15, 15, 16));
        asm::push_param_stack(p, 15);
        asm::next(p);
    });

    define_builtin_or_panic(p, (false, false), ">=".as_bytes(), |p| {
        asm::pop_param_stack(p, 15);
        asm::pop_param_stack(p, 16);
        p.inst(I::slt(17, 15, 16));
        p.inst(I::eql(18, 16, 15));
        p.inst(I::or(15, 17, 18));
        asm::push_param_stack(p, 15);
        asm::next(p);
    });

    ////////////////////////////////////////////////////////////////////////////
    // Bitwise arithmetic
    ////////////////////////////////////////////////////////////////////////////

    define_builtin_or_panic(p, (false, false), "and".as_bytes(), |p| {
        asm::pop_param_stack(p, 15);
        asm::pop_param_stack(p, 16);
        p.inst(I::and(15, 15, 16));
        asm::push_param_stack(p, 15);
        asm::next(p);
    });

    define_builtin_or_panic(p, (false, false), "or".as_bytes(), |p| {
        asm::pop_param_stack(p, 15);
        asm::pop_param_stack(p, 16);
        p.inst(I::or(15, 15, 16));
        asm::push_param_stack(p, 15);
        asm::next(p);
    });

    define_builtin_or_panic(p, (false, false), "xor".as_bytes(), |p| {
        asm::pop_param_stack(p, 15);
        asm::pop_param_stack(p, 16);
        p.inst(I::xor(15, 15, 16));
        asm::push_param_stack(p, 15);
        asm::next(p);
    });

    define_builtin_or_panic(p, (false, false), "invert".as_bytes(), |p| {
        asm::pop_param_stack(p, 15);
        p.inst(I::xori(16, 0, 0xFFFF));
        p.inst(I::xorui(16, 16, 0xFFFF));
        p.inst(I::xor(15, 15, 16));
        asm::push_param_stack(p, 15);
        asm::next(p);
    });

    ////////////////////////////////////////////////////////////////////////////
    // Oddities
    ////////////////////////////////////////////////////////////////////////////

    define_builtin_or_panic(p, (false, false), "exit".as_bytes(), |p| {
        asm::pop_return_stack(p, 11);
        asm::next(p);
    });

    define_builtin_or_panic(p, (false, false), "lit".as_bytes(), |p| {
        p.inst(I::ldw(15, 11, 0));
        asm::push_param_stack(p, 15);
        p.inst(I::addi(11, 11, 4));
        asm::next(p);
    });

    ////////////////////////////////////////////////////////////////////////////
    // Memory
    ////////////////////////////////////////////////////////////////////////////

    define_builtin_or_panic(p, (false, false), "!32".as_bytes(), |p| {
        asm::pop_param_stack(p, 15); // address to store at.
        asm::pop_param_stack(p, 16); // data to store there.
        p.inst(I::stw(16, 15, 0));
        asm::next(p);
    });

    define_builtin_or_panic(p, (false, false), "@32".as_bytes(), |p| {
        asm::pop_param_stack(p, 15); // address to fetch.
        p.inst(I::ldw(16, 15, 0));
        asm::push_param_stack(p, 16);
        asm::next(p);
    });

    define_builtin_or_panic(p, (false, false), "!16".as_bytes(), |p| {
        asm::pop_param_stack(p, 15); // address to store at.
        asm::pop_param_stack(p, 16); // data to store there.
        p.inst(I::sthw(16, 15, 0));
        asm::next(p);
    });

    define_builtin_or_panic(p, (false, false), "@16".as_bytes(), |p| {
        asm::pop_param_stack(p, 15); // address to fetch.
        p.inst(I::ldhwu(16, 15, 0));
        asm::push_param_stack(p, 16);
        asm::next(p);
    });

    define_builtin_or_panic(p, (false, false), "!8".as_bytes(), |p| {
        asm::pop_param_stack(p, 15); // address to store at.
        asm::pop_param_stack(p, 16); // data to store there.
        p.inst(I::stb(16, 15, 0));
        asm::next(p);
    });

    define_builtin_or_panic(p, (false, false), "@8".as_bytes(), |p| {
        asm::pop_param_stack(p, 15); // address to fetch.
        p.inst(I::ldbu(16, 15, 0));
        asm::push_param_stack(p, 16);
        asm::next(p);
    });

    define_builtin_or_panic(p, (false, false), "+!32".as_bytes(), |p| {
        asm::pop_param_stack(p, 15); // address to store at.
        asm::pop_param_stack(p, 16); // amount to add.
        p.inst(I::ldw(17, 15, 0));
        p.inst(I::add(17, 17, 16));
        p.inst(I::stw(17, 15, 0));
        asm::next(p);
    });

    define_builtin_or_panic(p, (false, false), "-!32".as_bytes(), |p| {
        asm::pop_param_stack(p, 15); // address to store at.
        asm::pop_param_stack(p, 16); // amount to sub.
        p.inst(I::ldw(17, 15, 0));
        p.inst(I::sub(17, 17, 16));
        p.inst(I::stw(17, 15, 0));
        asm::next(p);
    });

    ////////////////////////////////////////////////////////////////////////////
    // Variables
    ////////////////////////////////////////////////////////////////////////////

    let state_addr = define_variable_or_panic(p, (false, false), "state".as_bytes(), None);
    let mem_addr = define_variable_or_panic(p, (false, false), "mem".as_bytes(), None);
    let latest_addr = define_variable_or_panic(p, (false, false), "latest".as_bytes(), None);
    let stack_start_addr = define_variable_or_panic(
        p,
        (false, false),
        "stack_start".as_bytes(),
        Some(enigma::STACK_BEGINNING),
    );
    let number_base_addr =
        define_variable_or_panic(p, (false, false), "number_base".as_bytes(), Some(10));

    p.builder.write_word(state_addr, 0);
    p.builder.write_word(mem_addr, p.head.0);
    p.builder.write_word(latest_addr, p.latest.0);
    p.builder
        .write_word(stack_start_addr, enigma::STACK_BEGINNING);
    p.builder.write_word(number_base_addr, 10);
}

fn main() {
    let mut p = Putter::new();
    define_builtin_words(&mut p);
    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();
    p.builder.dump_chunks(&mut stdout).expect("io error");
}
