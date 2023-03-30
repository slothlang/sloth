#![feature(macro_metavar_expr)]
#![allow(dead_code)]
#![warn(
    clippy::wildcard_imports,
    clippy::string_add,
    clippy::string_add_assign,
    clippy::manual_ok_or,
    unused_lifetimes
)]

macro_rules! instructions {
    ( $( $opcode:literal $name:ident [ $( $v_type:ident ),* ] $doc:literal ),* ) => {
        #[repr(u8)]
        enum Instruction {
            $(
                #[doc = $doc]
                $name ( $( $v_type ),* ) = $opcode
            ),*
        }

        impl Instruction {
            fn opcode(&self) -> u8 {
                match self {
                    $(
                        Self::$name ( $( _ ${ignore(v_type)} ),* ) => $opcode
                    ),*
                }
            }

            fn from_bytecode(bytecode: &[u8]) -> Option<Self> {
                if bytecode.is_empty() {
                    return None;
                }

                let opcode = bytecode[0];
                let instruction = match opcode {
                    $(
                        $opcode => {
                            // TODO: Get the actual values
                            Some(Self::$name ( $( 0 ${ignore(v_type)} ),* ))
                        }
                    ),*,
                    _ => None,
                };

                instruction
            }
        }
    }
}

instructions! {
    0x00 Constant   [u64]   "Push a constant value onto the stack",

    0x01 Pop        []      "Pop a value from the stack",
    0x02 Dup        []      "Duplicate a value on the stack",

    0x10 Add        []      "Add the last 2 values on the stack",
    0x11 Sub        []      "Subtract the last 2 values on the stack",
    0x12 Mul        []      "Multiply the last 2 values on the stack",
    0x13 Div        []      "Divide the last 2 values on the stack",
    0x14 Mod        []      "Modulo the last 2 values on the stack"
}
