#![allow(dead_code)]
#![warn(
    clippy::wildcard_imports,
    clippy::string_add,
    clippy::string_add_assign,
    clippy::manual_ok_or,
    unused_lifetimes
)]

use sloth_bytecode::Instruction;

pub struct Chunk {
    code: Vec<u8>,
    constants: Vec<Data>,
}

pub struct VM {
    vm_return: Option<Data>,
    stack: Stack,
}

impl VM {
    fn new() -> Self {
        Self {
            vm_return: None,
            stack: Stack::default(),
        }
    }

    fn run(&mut self, chunk: &Chunk) {
        let mut pointer = 0;

        loop {
            let instruction = Instruction::disassemble(&chunk.code, &mut pointer);

            match instruction {
                Instruction::Constant(idx) => {
                    let value = chunk.constants[idx as usize];
                    self.stack.push(value);
                }
                Instruction::Load(_) => todo!(),
                Instruction::Push(_) => todo!(),
                Instruction::Dup => {
                    let value = self.stack.pop();
                    self.stack.push(value);
                    self.stack.push(value);
                }
                Instruction::Pop => {
                    self.stack.pop();
                }
                Instruction::Add => {
                    let value = match self.stack.pop2() {
                        (Data::Integer(lhs), Data::Integer(rhs)) => Data::Integer(lhs + rhs),
                        (Data::Float(lhs), Data::Float(rhs)) => Data::Float(lhs + rhs),
                        _ => panic!(),
                    };

                    self.stack.push(value);
                }
                Instruction::Sub => {
                    let value = match self.stack.pop2() {
                        (Data::Integer(lhs), Data::Integer(rhs)) => Data::Integer(lhs - rhs),
                        (Data::Float(lhs), Data::Float(rhs)) => Data::Float(lhs - rhs),
                        _ => panic!(),
                    };

                    self.stack.push(value);
                }
                Instruction::Mul => {
                    let value = match self.stack.pop2() {
                        (Data::Integer(lhs), Data::Integer(rhs)) => Data::Integer(lhs * rhs),
                        (Data::Float(lhs), Data::Float(rhs)) => Data::Float(lhs * rhs),
                        _ => panic!(),
                    };

                    self.stack.push(value);
                }
                Instruction::Div => {
                    let value = match self.stack.pop2() {
                        (Data::Integer(lhs), Data::Integer(rhs)) => Data::Integer(lhs / rhs),
                        (Data::Float(lhs), Data::Float(rhs)) => Data::Float(lhs / rhs),
                        _ => panic!(),
                    };

                    self.stack.push(value);
                }
                Instruction::Mod => {
                    let value = match self.stack.pop2() {
                        (Data::Integer(lhs), Data::Integer(rhs)) => Data::Integer(lhs % rhs),
                        (Data::Float(lhs), Data::Float(rhs)) => Data::Float(lhs % rhs),
                        _ => panic!(),
                    };

                    self.stack.push(value);
                }
                Instruction::VMReturn => {
                    let value = self.stack.pop();
                    self.vm_return = Some(value);
                    break;
                }
                Instruction::VMExit => break,
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Data {
    Integer(i128),
    Float(f64),
    Bool(bool),
    Empty,
}

const STACK_SIZE: usize = 1024;

#[derive(Debug)]
pub struct Stack {
    pointer: usize,
    stack: [Data; STACK_SIZE],
}

impl Default for Stack {
    fn default() -> Self {
        Self {
            pointer: Default::default(),
            stack: [Data::Empty; STACK_SIZE],
        }
    }
}

impl Stack {
    #[inline(always)]
    pub fn push(&mut self, value: Data) {
        if self.pointer >= STACK_SIZE {
            panic!("Stack overflow");
        }

        self.stack[self.pointer] = value;
        self.pointer += 1;
    }

    #[inline(always)]
    pub fn pop(&mut self) -> Data {
        if self.pointer == 0 {
            panic!("Stack underflow");
        }

        self.pointer -= 1;
        self.stack[self.pointer]
    }

    #[inline(always)]
    pub fn pop2(&mut self) -> (Data, Data) {
        (self.pop(), self.pop())
    }
}

#[cfg(test)]
mod tests {
    use crate::{Chunk, Data, VM};

    #[test]
    fn arithmetic_ops() {
        let mut vm = VM::new();

        // Addition
        vm.run(&Chunk {
            code: vec![
                0x00, 0, 0, 0, 0, 0, 0, 0, 0,    // Load constant from 0
                0x10, // Duplicate
                0x20, // Add
                0xF0, // Return VM
            ],
            constants: vec![Data::Integer(7)],
        });

        let add1 = vm.vm_return;

        vm.run(&Chunk {
            code: vec![
                0x00, 0, 0, 0, 0, 0, 0, 0, 0, // Load constant from 0
                0x00, 0, 0, 0, 0, 0, 0, 0, 1,    // Load constant from 1
                0x20, // Add
                0xF0, // Return VM
            ],
            constants: vec![Data::Integer(2), Data::Integer(11)],
        });

        let add2 = vm.vm_return;

        // Subtraction
        vm.run(&Chunk {
            code: vec![
                0x00, 0, 0, 0, 0, 0, 0, 0, 0,    // Load constant from 0
                0x10, // Duplicate
                0x21, // Subtraction
                0xF0, // Return VM
            ],
            constants: vec![Data::Integer(7)],
        });

        let sub1 = vm.vm_return;

        vm.run(&Chunk {
            code: vec![
                0x00, 0, 0, 0, 0, 0, 0, 0, 0, // Load constant from 0
                0x00, 0, 0, 0, 0, 0, 0, 0, 1,    // Load constant from 1
                0x21, // Subtraction
                0xF0, // Return VM
            ],
            constants: vec![Data::Integer(2), Data::Integer(11)],
        });

        let sub2 = vm.vm_return;

        // Multiplication
        vm.run(&Chunk {
            code: vec![
                0x00, 0, 0, 0, 0, 0, 0, 0, 0,    // Load constant from 0
                0x10, // Duplicate
                0x22, // Multiplication
                0xF0, // Return VM
            ],
            constants: vec![Data::Integer(7)],
        });

        let mul1 = vm.vm_return;

        vm.run(&Chunk {
            code: vec![
                0x00, 0, 0, 0, 0, 0, 0, 0, 0, // Load constant from 0
                0x00, 0, 0, 0, 0, 0, 0, 0, 1,    // Load constant from 1
                0x22, // Multiplication
                0xF0, // Return VM
            ],
            constants: vec![Data::Integer(2), Data::Integer(11)],
        });

        let mul2 = vm.vm_return;

        // Division
        vm.run(&Chunk {
            code: vec![
                0x00, 0, 0, 0, 0, 0, 0, 0, 0,    // Load constant from 0
                0x10, // Duplicate
                0x23, // Division
                0xF0, // Return VM
            ],
            constants: vec![Data::Integer(7)],
        });

        let div1 = vm.vm_return;

        vm.run(&Chunk {
            code: vec![
                0x00, 0, 0, 0, 0, 0, 0, 0, 0, // Load constant from 0
                0x00, 0, 0, 0, 0, 0, 0, 0, 1,    // Load constant from 1
                0x23, // Division
                0xF0, // Return VM
            ],
            constants: vec![Data::Integer(2), Data::Integer(11)],
        });

        let div2 = vm.vm_return;

        assert_eq!(add1, Some(Data::Integer(14)));
        assert_eq!(add2, Some(Data::Integer(13)));

        assert_eq!(sub1, Some(Data::Integer(0)));
        assert_eq!(sub2, Some(Data::Integer(9)));

        assert_eq!(mul1, Some(Data::Integer(49)));
        assert_eq!(mul2, Some(Data::Integer(22)));

        assert_eq!(div1, Some(Data::Integer(1)));
        assert_eq!(div2, Some(Data::Integer(5)));
    }
}
