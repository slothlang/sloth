#![allow(dead_code)]
#![warn(
    clippy::wildcard_imports,
    clippy::string_add,
    clippy::string_add_assign,
    clippy::manual_ok_or,
    unused_lifetimes
)]

use sloth_bytecode_macros::instructions;

pub enum Error {
    UnknownOpcode(u8),
    InvalidArguments,
    Eof,
}

instructions! {
    Instructions;

    0x00 Constant   [u64]   "Push a constant value onto the stack",
    0x01 Load       [u64]   "Load a value from a variable",
    0x02 Push       [u64]   "Push a value to a variable",

    0x10 Dup        []      "Duplicate a value on the stack",
    0x11 Pop        []      "Pop a value from the stack",

    0x20 Add        []      "Add the last 2 values on the stack",
    0x21 Sub        []      "Subtract the last 2 values on the stack",
    0x22 Mul        []      "Multiply the last 2 values on the stack",
    0x23 Div        []      "Divide the last 2 values on the stack",
    0x24 Mod        []      "Modulo the last 2 values on the stack",

    0xF0 Print      []      "[DEBUG] Pop value from stack and print it"
}

#[cfg(test)]
mod tests {
    use crate::Instructions;

    #[test]
    #[rustfmt::skip]
    fn decompile_basic_instructions() {
        let code = [
            // Load constant 0
            0x00, 0, 0, 0, 0, 0, 0, 0, 0, 
            // Pop, Dup, Add, Sub, Mul, Div, Mod
            0x10, 0x11, 0x20, 0x21, 0x22, 0x23, 0x24,
        ];

        let mut offset = 0;

        assert_eq!(Instructions::disassemble(&code, &mut offset), Instructions::Constant(0));
        assert_eq!(Instructions::disassemble(&code, &mut offset), Instructions::Dup);
        assert_eq!(Instructions::disassemble(&code, &mut offset), Instructions::Pop);
        assert_eq!(Instructions::disassemble(&code, &mut offset), Instructions::Add);
        assert_eq!(Instructions::disassemble(&code, &mut offset), Instructions::Sub);
        assert_eq!(Instructions::disassemble(&code, &mut offset), Instructions::Mul);
        assert_eq!(Instructions::disassemble(&code, &mut offset), Instructions::Div);
        assert_eq!(Instructions::disassemble(&code, &mut offset), Instructions::Mod);
    }
}
