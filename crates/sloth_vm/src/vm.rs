use std::mem::MaybeUninit;

use sloth_bytecode::Opcode;

use crate::value::{Function, Object, ObjectType, Primitive};
use crate::{ObjectMap, Stack};

#[derive(Clone, Copy)]
pub struct CallFrame {
    pointer: usize,
    stack_offset: usize,
    function: *const Function, // TODO: Safety
}

impl CallFrame {
    fn new(stack_offset: usize, function: &Function) -> Self {
        Self {
            pointer: 0,
            stack_offset,
            function: function as *const _,
        }
    }

    #[inline]
    fn function(&self) -> &Function {
        unsafe { &*self.function }
    }
}

const CALL_STACK_SIZE: usize = 1024;

pub struct CallStack {
    top: usize,
    frames: [MaybeUninit<CallFrame>; CALL_STACK_SIZE],
}

impl Default for CallStack {
    fn default() -> Self {
        Self {
            top: 0,
            frames: [MaybeUninit::uninit(); CALL_STACK_SIZE],
        }
    }
}

impl CallStack {
    fn push(&mut self, frame: CallFrame) {
        self.frames[self.top].write(frame);
        self.top += 1;
    }

    fn pop(&mut self) {
        self.top -= 1;
    }

    fn peek(&self) -> &CallFrame {
        unsafe { self.frames[self.top - 1].assume_init_ref() }
    }

    fn peek_mut(&mut self) -> &mut CallFrame {
        unsafe { self.frames[self.top - 1].assume_init_mut() }
    }
}

pub struct VM {
    stack: Stack,
    call_stack: CallStack,
    objects: ObjectMap,
}

impl Default for VM {
    fn default() -> Self {
        Self::init(ObjectMap::default())
    }
}

impl VM {
    pub fn init(objects: ObjectMap) -> Self {
        Self {
            stack: Stack::default(),
            call_stack: CallStack::default(),
            objects,
        }
    }

    pub fn new(objects: ObjectMap, mut root: Function) -> Self {
        let mut this = Self::init(objects);

        // Allocating the root function
        root.chunk.code.push(Opcode::Halt as u8);
        this.call_stack.push(CallFrame::new(0, &root));
        this.objects
            .allocate(Object::new(ObjectType::Function(root)));

        this
    }

    pub fn step(&mut self) -> bool {
        use Primitive::*;

        let opcode = self.read_u8();

        match Opcode::from_u8(opcode) {
            Opcode::Constant => {
                let idx = self.read_u16() as usize;
                let value = self.call_stack.peek().function().chunk.constants[idx];

                self.stack.push(value);
            }
            Opcode::Dup => {
                let value = self.stack.pop();
                self.stack.push(value);
                self.stack.push(value);
            }
            Opcode::Pop => {
                self.stack.pop();
            }
            Opcode::GetLocal => {
                let idx = self.read_u16() as usize;
                let value = self.stack[self.call_stack.peek().stack_offset + idx];

                self.stack.push(value);
            }
            Opcode::SetLocal => {
                let idx = self.read_u16() as usize;
                let value = self.stack.pop();

                self.stack[self.call_stack.peek().stack_offset + idx] = value;
            }

            Opcode::Add => {
                let value = match self.stack.pop2() {
                    (Integer(lhs), Integer(rhs)) => Integer(lhs + rhs),
                    (Float(lhs), Float(rhs)) => Float(lhs + rhs),
                    _ => panic!(),
                };

                self.stack.push(value);
            }
            Opcode::Sub => {
                let value = match self.stack.pop2() {
                    (Integer(lhs), Integer(rhs)) => Integer(lhs - rhs),
                    (Float(lhs), Float(rhs)) => Float(lhs - rhs),
                    _ => panic!(),
                };

                self.stack.push(value);
            }
            Opcode::Mul => {
                let value = match self.stack.pop2() {
                    (Integer(lhs), Integer(rhs)) => Integer(lhs * rhs),
                    (Float(lhs), Float(rhs)) => Float(lhs * rhs),
                    _ => panic!(),
                };

                self.stack.push(value);
            }
            Opcode::Div => {
                let value = match self.stack.pop2() {
                    (Integer(_), Integer(0)) => panic!("Divide by 0"),
                    (Integer(lhs), Integer(rhs)) => Integer(lhs / rhs),
                    (Float(lhs), Float(rhs)) => Float(lhs / rhs),
                    _ => panic!(),
                };

                self.stack.push(value);
            }
            Opcode::Mod => {
                let value = match self.stack.pop2() {
                    (Integer(lhs), Integer(rhs)) => Integer(lhs % rhs),
                    (Float(lhs), Float(rhs)) => Float(lhs % rhs),
                    _ => panic!(),
                };

                self.stack.push(value);
            }

            Opcode::Call => {
                let Primitive::Object(ptr) = self.stack.pop() else {
                    panic!("Last element on stack was not an object");
                };

                let Some(obj) = self.objects.get(ptr as usize) else {
                    panic!("Pointer referenced nothing");
                };

                let ObjectType::Function(function) = &obj.typ else {
                    panic!("Object was not a function");
                };

                // Push the function onto the call stack
                self.call_stack.push(CallFrame::new(0, function));
            }

            Opcode::Return => {
                // TODO: Return values

                self.call_stack.pop();
            }

            Opcode::Halt => return false,

            _ => unimplemented!(),
        }

        true
    }

    pub fn run(&mut self) {
        while self.step() {}
    }

    fn call(&mut self, function: &Function) {
        self.call_stack.push(CallFrame {
            pointer: 0,
            stack_offset: self.stack.top - 1 - (function.arity as usize),
            function,
        });
    }

    #[inline(always)]
    fn read_u8(&mut self) -> u8 {
        let frame = self.call_stack.peek_mut();
        let function = frame.function();
        let byte = function.chunk.code[frame.pointer];
        frame.pointer += 1;
        byte
    }

    #[inline(always)]
    fn read_u16(&mut self) -> u16 {
        let frame = self.call_stack.peek_mut();
        let chunk = &frame.function().chunk;

        let bytes = (chunk.code[frame.pointer], chunk.code[frame.pointer + 1]);

        frame.pointer += 2;

        ((bytes.0 as u16) << 8) + (bytes.1 as u16)
    }
}

#[cfg(test)]
mod tests {
    use crate::value::{Function, Object, ObjectType, Primitive};
    use crate::{Chunk, ObjectMap, VM};

    #[test]
    fn arithmetic_ops() {
        // Addition
        let mut vm = VM::new(
            ObjectMap::default(),
            Function::root(Chunk {
                constants: vec![Primitive::Integer(7)],
                code: vec![
                    0x00, 0, 0,    // Load constant from 0
                    0x10, // Duplicate
                    0x20, // Add
                    0xE0,
                ],
            }),
        );

        vm.run();
        assert_eq!(vm.stack.peek(), Primitive::Integer(14));

        let mut vm = VM::new(
            ObjectMap::default(),
            Function::root(Chunk {
                constants: vec![Primitive::Integer(2), Primitive::Integer(11)],
                code: vec![
                    0x00, 0, 0, // Load constant from 0
                    0x00, 0, 1,    // Load constant from 1
                    0x20, // Add
                    0xE0,
                ],
            }),
        );

        vm.run();
        assert_eq!(vm.stack.peek(), Primitive::Integer(13));
    }

    #[test]
    fn allocation() {
        let mut vm = VM::new(
            ObjectMap::from(vec![
                Object::new(ObjectType::String("Hello World!".to_owned())),
                Object::new(ObjectType::String("Hello Slothlang!".to_owned())),
                Object::new(ObjectType::Function(Function {
                    name: Some("foo".to_string()),
                    chunk: Chunk {
                        constants: vec![Primitive::Integer(7)],
                        code: vec![0x00, 0, 0, 0x10, 0x20, 0x52],
                    },
                    arity: 0,
                })),
            ]),
            Function::root(Chunk {
                constants: vec![
                    Primitive::Object(0),
                    Primitive::Object(1),
                    Primitive::Object(2),
                ],
                code: vec![
                    0x00, 0, 0, // Load constant from 0
                    0x00, 0, 1, // Load constant from 1
                    0x00, 0, 2, // Load constant from 2
                    0x50, 0xE0,
                ],
            }),
        );

        vm.run();

        assert_eq!(vm.stack.peek(), Primitive::Integer(14));
    }
}
