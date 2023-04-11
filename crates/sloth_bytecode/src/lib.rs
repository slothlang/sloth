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
    Instruction;

    0x00 Constant   [u32]   "Push a constant value onto the stack",
    0x01 Load       [u32]   "Load a value from a variable",
    0x02 Push       [u32]   "Push a value to a variable",

    0x10 Dup        []      "Duplicate a value on the stack",
    0x11 Del        []      "Delete a value from the stack",

    0x20 Add        []      "Add the last 2 values on the stack",
    0x21 Sub        []      "Subtract the last 2 values on the stack",
    0x22 Mul        []      "Multiply the last 2 values on the stack",
    0x23 Div        []      "Divide the last 2 values on the stack",
    0x24 Mod        []      "Modulo the last 2 values on the stack",

    0x30 Eq         []      "Check if the last 2 values on the stack are equal",
    0x31 Ne         []      "Check if the last 2 values on the stack are not equal",

    0x40 Jmp        [u32]   "Jump to a specific point in the program",
    0x41 JmpIf      [u32]   "Jump to a specific point in the program if true is on the stack",

    0xE0 Hlt        []      "Halt the program",
    0xE1 Exit       []      "Exit the program",

    0xF0 VMReturn   []      "[DEBUG] Pop value from stack and return it from the program",
}

#[cfg(test)]
mod tests {
    use crate::Instruction;

    #[test]
    #[rustfmt::skip]
    fn decompile_basic_instructions() {
        let code = [
            // Load constant 0
            0x00, 0, 0, 0, 0,  
            // Pop, Del, Add, Sub, Mul, Div, Mod
            0x10, 0x11, 0x20, 0x21, 0x22, 0x23, 0x24,
        ];

        let mut offset = 0;

        assert_eq!(Instruction::disassemble(&code, &mut offset), Instruction::Constant(0));
        assert_eq!(Instruction::disassemble(&code, &mut offset), Instruction::Dup);
        assert_eq!(Instruction::disassemble(&code, &mut offset), Instruction::Del);
        assert_eq!(Instruction::disassemble(&code, &mut offset), Instruction::Add);
        assert_eq!(Instruction::disassemble(&code, &mut offset), Instruction::Sub);
        assert_eq!(Instruction::disassemble(&code, &mut offset), Instruction::Mul);
        assert_eq!(Instruction::disassemble(&code, &mut offset), Instruction::Div);
        assert_eq!(Instruction::disassemble(&code, &mut offset), Instruction::Mod);
    }
}
