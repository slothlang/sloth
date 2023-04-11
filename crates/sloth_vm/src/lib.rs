#![allow(dead_code)]
#![warn(
    clippy::wildcard_imports,
    clippy::string_add,
    clippy::string_add_assign,
    clippy::manual_ok_or,
    unused_lifetimes
)]

pub mod native;
pub mod obj;
pub mod vm;

pub use crate::vm::VM;

pub struct Chunk {
    constants: Vec<Data>,
    code: Vec<u8>,
}

pub struct Function {
    chunk: Chunk,
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

pub struct ObjectMap {}
