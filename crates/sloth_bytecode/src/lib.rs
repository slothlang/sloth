#![allow(dead_code)]
#![warn(
    clippy::wildcard_imports,
    clippy::string_add,
    clippy::string_add_assign,
    clippy::manual_ok_or,
    unused_lifetimes
)]

pub enum Error {
    UnknownOpcode(u8),
    InvalidArguments,
    Eof,
}

macro_rules! opcodes {
    ( $( $code:literal $name:ident $docs:literal ),* ) => {
        #[repr(u8)]
        #[derive(Debug, Clone, Copy, Eq, PartialEq)]
        pub enum Opcode {
            $(
                #[doc = $docs]
                $name = $code
            ),*
        }

        impl Opcode {
            pub fn into_u8(self) -> u8 {
                self as u8
            }

            pub fn from_u8(value: u8) -> Opcode {
                match value {
                    $( $code => Self:: $name , )*
                    _ => panic!("Invalid opcode"),
                }
            }
        }
    };
}

opcodes! {
    0x00 Constant   "Push a constant value onto the stack",
    0x01 Load       "Load a value from a variable",
    0x02 Push       "Push a value to a variable",

    0x10 Dup        "Duplicate a value on the stack",
    0x11 Pop        "Pop a value from the stack",

    0x12 GetGlobal  "Get a global value",
    0x13 SetGlobal  "Set a global value",
    0x14 GetLocal   "Get a local value",
    0x15 SetLocal   "Set a local value",

    0x20 Add        "Add the last 2 values on the stack",
    0x21 Sub        "Subtract the last 2 values on the stack",
    0x22 Mul        "Multiply the last 2 values on the stack",
    0x23 Div        "Divide the last 2 values on the stack",
    0x24 Mod        "Modulo the last 2 values on the stack",

    0x30 Eq         "Check if the last 2 values on the stack are equal",
    0x31 Ne         "Check if the last 2 values on the stack are not equal",

    0x40 Jump       "Jump to a specific point in the program",
    0x41 JumpIf     "Jump to a specific point in the program if true is on the stack",

    0x50 Call       "Call function on stack",
    0x51 CallNative "Call native function",
    0x52 Return     "Return from function on stack",

    0xE0 Halt       "Halt the program",

    0xF0 VMReturn   "[DEBUG] Pop value from stack and return it fromthe program",
    0xF1 VMPrint    "[DEBUG] Print value to console"
}
