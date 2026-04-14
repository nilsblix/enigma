#![allow(unused)]

use enigma::STACK_BEGINNING;

const RETURN_STACK_TOP: u32 = STACK_BEGINNING - 0x0001_0000;
const CONSOLE_CHAR_OFFSET: i16 = 0;
const CONSOLE_SIGNED_OFFSET: i16 = 4;
const CONSOLE_UNSIGNED_OFFSET: i16 = 8;

enum Definition {
    Macro { words: Vec<Word> },
    Builtin { asm: Vec<enigma::is::Instruction> },
}

struct Word {
    link: Option<Box<Word>>,
    flags_and_name: u8,
    name: Vec<u8>,
    definition: Definition,
}

fn tokenize(source: &str) -> Vec<&str> {
    source.split_whitespace().collect()
}

fn find_word<'a, 'b>(dict: Option<&'a Word>, word: &'b str) -> Option<&'a Word> {
    if dict.is_none() {
        return None;
    }

    let name = word.as_bytes();
    let mut node = dict.unwrap();
    loop {
        if node.name.as_slice() == name {
            return Some(node);
        }

        if let Some(next) = node.link.as_ref() {
            return Some(next.as_ref());
        }

        return None;
    }
}

fn append_to_link(dict: Word, mut node: Word) -> Word {
    node.link = Some(Box::new(dict));
    node
}

fn main() {
    let forth = "12 13 + .";
    let tokens = tokenize(forth);

    let mut dict = None;
    for &incoming in tokens.iter() {
        if let Some(word) = find_word(dict, incoming) {}
    }
}
