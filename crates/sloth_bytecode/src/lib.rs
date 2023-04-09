#![allow(dead_code)]
#![warn(
    clippy::wildcard_imports,
    clippy::string_add,
    clippy::string_add_assign,
    clippy::manual_ok_or,
    unused_lifetimes
)]

use sloth_bytecode_macros::instructions;

pub struct Chunk {
    pub code: Vec<u8>,
    pub constants: Vec<u64>,
}

pub enum Error {
    UnknownOpcode(u8),
    InvalidArguments,
    Eof,
}

instructions! {
    Instructions;

    0x00 Constant   [u64]   "Push a constant value onto the stack",

    0x01 Pop        []      "Pop a value from the stack",
    0x02 Dup        []      "Duplicate a value on the stack",

    0x10 Add        []      "Add the last 2 values on the stack",
    0x11 Sub        []      "Subtract the last 2 values on the stack",
    0x12 Mul        []      "Multiply the last 2 values on the stack",
    0x13 Div        []      "Divide the last 2 values on the stack",
    0x14 Mod        []      "Modulo the last 2 values on the stack"
}

#[cfg(test)]
mod tests {
    use crate::{Chunk, Instructions};

    #[test]
    #[rustfmt::skip]
    fn decompile_basic_instructions() {
        let code = vec![
            // Load constant 0
            0x00, 0, 0, 0, 0, 0, 0, 0, 0, 
            // Pop, Dup, Add, Sub, Mul, Div, Mod
            0x01, 0x02, 0x10, 0x11, 0x12, 0x13, 0x14,
        ];

        let chunk = Chunk {
            code,
            constants: Vec::new(),
        };

        let mut offset = 0;

        assert_eq!(Instructions::disassemble(&chunk, &mut offset), Instructions::Constant(0));
        assert_eq!(Instructions::disassemble(&chunk, &mut offset), Instructions::Pop);
        assert_eq!(Instructions::disassemble(&chunk, &mut offset), Instructions::Dup);
        assert_eq!(Instructions::disassemble(&chunk, &mut offset), Instructions::Add);
        assert_eq!(Instructions::disassemble(&chunk, &mut offset), Instructions::Sub);
        assert_eq!(Instructions::disassemble(&chunk, &mut offset), Instructions::Mul);
        assert_eq!(Instructions::disassemble(&chunk, &mut offset), Instructions::Div);
        assert_eq!(Instructions::disassemble(&chunk, &mut offset), Instructions::Mod);
    }
}
