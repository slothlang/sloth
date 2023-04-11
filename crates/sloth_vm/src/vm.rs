use sloth_bytecode::Instruction;

use crate::{Chunk, Data, Stack};

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

    fn execute(&mut self) {
        loop {
            self.execute_once();
        }
    }

    fn execute_once(&mut self) {
        //
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
                Instruction::Del => {
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
                Instruction::Hlt => break,
                Instruction::Exit => break,
                Instruction::VMReturn => {
                    let value = self.stack.pop();
                    self.vm_return = Some(value);
                    break;
                }
                _ => unimplemented!(),
            }
        }
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
            constants: vec![Data::Integer(7)],
            code: vec![
                0x00, 0, 0, 0, 0,    // Load constant from 0
                0x10, // Duplicate
                0x20, // Add
                0xF0, // Return VM
            ],
        });

        assert_eq!(vm.vm_return, Some(Data::Integer(14)));

        vm.run(&Chunk {
            constants: vec![Data::Integer(2), Data::Integer(11)],
            code: vec![
                0x00, 0, 0, 0, 0, // Load constant from 0
                0x00, 0, 0, 0, 1,    // Load constant from 1
                0x20, // Add
                0xF0, // Return VM
            ],
        });

        assert_eq!(vm.vm_return, Some(Data::Integer(13)));
    }
}
