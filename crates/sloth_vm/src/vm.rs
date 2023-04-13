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

            Opcode::Eq => {
                let value = match self.stack.pop2() {
                    (Integer(lhs), Integer(rhs)) => Bool(lhs == rhs),
                    (Float(lhs), Float(rhs)) => Bool(lhs == rhs),
                    (Bool(lhs), Bool(rhs)) => Bool(lhs == rhs),
                    (Object(lhs), Object(rhs)) => Bool(lhs == rhs),
                    (Empty, Empty) => Bool(true),
                    _ => Bool(false),
                };

                self.stack.push(value);
            }
            Opcode::Ne => {
                let value = match self.stack.pop2() {
                    (Integer(lhs), Integer(rhs)) => Bool(lhs != rhs),
                    (Float(lhs), Float(rhs)) => Bool(lhs != rhs),
                    (Bool(lhs), Bool(rhs)) => Bool(lhs != rhs),
                    (Object(lhs), Object(rhs)) => Bool(lhs != rhs),
                    (Empty, Empty) => Bool(false),
                    _ => Bool(false),
                };

                self.stack.push(value);
            }

            Opcode::Jump => {
                let to = self.read_u16();
                self.call_stack.peek_mut().pointer = to as usize;
            }
            Opcode::JumpIf => {
                let to = self.read_u16();
                let value = self.stack.pop();

                if let Bool(true) = value {
                    self.call_stack.peek_mut().pointer = to as usize;
                }
            }

            Opcode::Call => {
                let Primitive::Object(ptr) = self.stack.pop() else {
                    panic!("Last element on stack was not an object");
                };

                self.call(ptr as usize);
            }

            Opcode::Return => {
                self.call_return();
            }

            Opcode::Halt => return false,

            opcode => unimplemented!("Opcode {:?} unimplemented", opcode),
        }

        true
    }

    pub fn run(&mut self) {
        while self.step() {}
    }

    fn call(&mut self, ptr: usize) {
        let Some(obj) = self.objects.get(ptr) else {
            panic!("Pointer referenced nothing");
        };

        let ObjectType::Function(function) = &obj.typ else {
            panic!("Object was not a function");
        };

        // Add a callstack entry for the function
        let offset = self.stack.top - (function.arity as usize);
        self.call_stack.push(CallFrame::new(offset, function));
    }

    fn call_return(&mut self) {
        let function = self.call_stack.peek().function();
        let stack_offset = self.call_stack.peek().stack_offset;

        let return_value = if function.returns_value {
            Some(self.stack.pop())
        } else {
            None
        };

        self.stack.top = stack_offset;

        if let Some(return_value) = return_value {
            self.stack.push(return_value);
        }

        self.call_stack.pop();
    }

    fn unwind(&mut self) {
        unimplemented!("Implement unwinding for error handling");
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
    fn basic_function() {
        let mut vm = VM::new(
            ObjectMap::from(vec![Object::new(ObjectType::Function(Function {
                name: Some("add".to_string()),
                chunk: Chunk {
                    constants: vec![],
                    code: vec![0x14, 0, 0, 0x14, 0, 1, 0x20, 0x52],
                },
                arity: 2,
                returns_value: true,
            }))]),
            Function::root(Chunk {
                constants: vec![
                    Primitive::Integer(6),
                    Primitive::Integer(3),
                    Primitive::Object(0),
                ],
                code: vec![
                    0x00, 0, 0, // Load first function parameter from 0
                    0x00, 0, 1, // Load second function parameter from 1
                    0x00, 0, 2,    // Load function constant from 2
                    0x50, // Call function
                ],
            }),
        );

        vm.run();

        assert_eq!(vm.stack.peek(), Primitive::Integer(9));
    }

    #[test]
    fn fibonacci() {
        #[rustfmt::skip]
        let mut vm = VM::new(
            ObjectMap::default(),
            Function::root(Chunk {
                constants: vec![
                    Primitive::Integer(0),
                    Primitive::Integer(1),
                    Primitive::Integer(10),
                ],
                code: vec![
                    // Load variables
                    0x00, 0, 0,  // 0  Index
                    0x00, 0, 0,  // 3  Me
                    0x00, 0, 0,  // 6  Parent
                    0x00, 0, 1,  // 9  Grandparent

                    // Load parent and grandparent, sum them and put the value in me
                    0x14, 0, 2,  // 12
                    0x14, 0, 3,  // 15
                    0x20,        // 16
                    0x15, 0, 1,  // 19
                    
                    // Set grandparent to parent
                    0x14, 0, 2,  // 22
                    0x15, 0, 3,  // 25
                    
                    // Set parent to me
                    0x14, 0, 1,  // 28
                    0x15, 0, 2,  // 31

                    // Increment Index by 1
                    0x00, 0, 1,  // 34
                    0x14, 0, 0,  // 37 Index
                    0x20,        // 40
                    0x15, 0, 0,  // 41 Index

                    // Load me
                    0x14, 0, 1,  // 44
                    0xE0,        // 47
                    0x11,        // 48

                    // Repeat until Index is 9
                    0x00, 0, 2,  // 49 
                    0x14, 0, 0,  // 52 Index
                    0x31,        // 55
                    0x41, 0, 12, // 56
                ],
            }),
        );

        let mut values = Vec::new();
        for _ in 0..10 {
            vm.run();
            values.push(vm.stack.peek());
        }

        assert_eq!(&values, &[
            Primitive::Integer(1),
            Primitive::Integer(1),
            Primitive::Integer(2),
            Primitive::Integer(3),
            Primitive::Integer(5),
            Primitive::Integer(8),
            Primitive::Integer(13),
            Primitive::Integer(21),
            Primitive::Integer(34),
            Primitive::Integer(55),
        ]);
    }

    #[test]
    fn fibonacci_recursive() {
        #[rustfmt::skip]
        let mut vm = VM::new(
            ObjectMap::from(vec![Object::new(ObjectType::Function(Function {
                name: Some("fib".to_owned()),
                chunk: Chunk {
                    constants: vec![
                        Primitive::Object(0),
                        Primitive::Integer(0),
                        Primitive::Integer(1),
                        Primitive::Integer(2),
                    ],
                    code: vec![
                        0x14, 0, 0,  // 0
                        0x00, 0, 1,  // 3
                        0x31,        // 6
                        0x41, 0, 14, // 7
                        0x00, 0, 1,  // 10
                        0x52,        // 13

                        0x14, 0, 0,  // 14
                        0x00, 0, 2,  // 17
                        0x31,        // 20
                        0x41, 0, 28, // 21
                        0x00, 0, 2,  // 24
                        0x52,        // 27

                        // fib(n - 1)
                        0x00, 0, 2,  // 28
                        0x14, 0, 0,  // 31
                        0x21,        // 34
                        0x00, 0, 0,  // 35
                        0x50,        // 38
                        
                        // fib(n - 2)
                        0x00, 0, 3,  // 39
                        0x14, 0, 0,  // 42
                        0x21,        // 45
                        0x00, 0, 0,  // 46
                        0x50,        // 49

                        // add & return
                        0x20,        // 50
                        0x52,        // 51
                    ],
                },
                arity: 1,
                returns_value: true,
            }))]),
            Function::root(Chunk {
                constants: vec![
                    Primitive::Object(0),
                    Primitive::Integer(10),
                ],
                code: vec![
                    // Load n and the function and call it
                    0x00, 0, 1, // 0
                    0x00, 0, 0, // 3
                    0x50,       // 6
                ],
            }),
        );

        vm.run();

        assert_eq!(Primitive::Integer(55), vm.stack.peek());
    }
}
