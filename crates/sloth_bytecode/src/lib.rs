#![allow(dead_code)]
#![warn(
    clippy::wildcard_imports,
    clippy::string_add,
    clippy::string_add_assign,
    clippy::manual_ok_or,
    unused_lifetimes
)]

use std::io::Cursor;

use byteorder::ReadBytesExt;
// use sloth_bytecode_macros::instructions;

pub struct Chunk {
    pub code: Vec<u8>,
    pub constants: Vec<u64>,
}

// instructions! {
//     Instructions;
//
//     0x00 Constant   [u64]   "Push a constant value onto the stack",
//
//     0x01 Pop        []      "Pop a value from the stack",
//     0x02 Dup        []      "Duplicate a value on the stack",
//
//     0x10 Add        []      "Add the last 2 values on the stack",
//     0x11 Sub        []      "Subtract the last 2 values on the stack",
//     0x12 Mul        []      "Multiply the last 2 values on the stack",
//     0x13 Div        []      "Divide the last 2 values on the stack",
//     0x14 Mod        []      "Modulo the last 2 values on the stack"
// }

// impl Instructions {
//     fn disassemble(chunk: &Chunk, offset: &mut usize) {
//         //
//     }
//
//     fn assemble(chunk: &mut Chunk) {
//         //
//     }
// }

// #[test]
// fn test() {
//     let mut cursor = Cursor::new(vec![0, 1, 0, 0, 1, 0, 0, 0, 0]);
//     let instruction = Instructions::from_bytecode(&mut cursor);
//     println!("{instruction:?}");
//     assert!(1 == 0);
// }

// macro_rules! instructions {
//     ( $( $opcode:literal $name:ident [ $( $v_type:ident ),* ] $doc:literal
// ),* ) => {         #[repr(u8)]
//         enum Instruction {
//             $(
//                 #[doc = $doc]
//                 $name ( $( $v_type ),* ) = $opcode
//             ),*
//         }
//
//         impl Instruction {
//             fn opcode(&self) -> u8 {
//                 match self {
//                     $(
//                         Self::$name ( $( _ ${ignore(v_type)} ),* ) => $opcode
//                     ),*
//                 }
//             }
//
//             fn from_bytecode(bytecode: &[u8]) -> Option<Self> {
//                 if bstecode.is_empty() {
//                     return None;
//                 }
//
//                 let opcode = bytecode[0];
//                 let instruction = match opcode {
//                     $(
//                         $opcode => {
//                             // TODO: Get the actual values
//                             Some(Self::$name ( $( 0 ${ignore(v_type)} ),* ))
//                         }
//                     ),*,
//                     _ => None,
//                 };
//
//                 instruction
//             }
//         }
//     }
// }

// instructions! {
//     Instructions;
//
//     0x00 Constant   [u64]   "Push a constant value onto the stack",
//
//     0x01 Pop        []      "Pop a value from the stack",
//     0x02 Dup        []      "Duplicate a value on the stack",
//
//     0x10 Add        []      "Add the last 2 values on the stack",
//     0x11 Sub        []      "Subtract the last 2 values on the stack",
//     0x12 Mul        []      "Multiply the last 2 values on the stack",
//     0x13 Div        []      "Divide the last 2 values on the stack",
//     0x14 Mod        []      "Modulo the last 2 values on the stack"
// }

pub enum Error {
    UnknownOpcode(u8),
    InvalidArguments,
    Eof,
}

pub enum Instruction {
    Constant(u64),

    Pop(),
    Dup(),

    Add(),
    Sub(),
    Mul(),
    Div(),
    Mod(),
}

// fn parse_bytecode(pos: usize, bc: &[u8]) -> Result<Bytecode, BytecodeError> {
//     let Some(opcode) = bc.get(pos) else {
//         return Err(BytecodeError::Eof);
//     };
//
//     let instruction = match opcode {
//         0x00 => {
//             // let arg0: [u8; 8] = bc.get(1..1+size_of::<u64>()).unwrap();
//             let arg0 = u64::from_ne_bytes(arg0);
//         }
//         _ => return Err(BytecodeError::UnknownOpcode(opcode)),
//     }
//
//     todo!()
// }

fn parse_bytecode(cursor: &mut Cursor<&[u8]>) -> Result<Instruction, Error> {
    let Ok(opcode) = cursor.read_u8() else {
        return Err(Error::Eof);
    };

    let instruction = match opcode {
        0x00 => {
            let arg0 = cursor
                .read_u64::<byteorder::LittleEndian>()
                .map_err(|_| Error::InvalidArguments)?;

            Instruction::Constant(arg0)
        }
        _ => return Err(Error::UnknownOpcode(opcode)),
    };

    Ok(instruction)
}

// impl<T: Iterator<Item = u8>> TryFrom<T> for Bytecode {
//     type Error = BytecodeError;
//
//     fn try_from(value: T) -> Result<Self, Self::Error> {
//         todo!()
//         //
//     }
// }
