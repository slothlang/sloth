#![allow(dead_code)]
#![warn(
    clippy::wildcard_imports,
    clippy::string_add,
    clippy::string_add_assign,
    clippy::manual_ok_or,
    unused_lifetimes
)]

pub mod native;
pub mod sloth_std;
pub mod value;
pub mod vm;

use std::ops::{Index, IndexMut};

use value::{Object, ObjectType};

use crate::value::Primitive;
pub use crate::vm::VM;

#[derive(Default)]
pub struct Chunk {
    pub constants: Vec<Primitive>,
    pub code: Vec<u8>,
}

const STACK_SIZE: usize = 1024;

#[derive(Debug)]
pub struct Stack {
    stack: [Primitive; STACK_SIZE],
    top: usize,
}

impl Default for Stack {
    fn default() -> Self {
        Self {
            top: Default::default(),
            stack: [Primitive::Empty; STACK_SIZE],
        }
    }
}

impl Stack {
    #[inline(always)]
    pub fn push(&mut self, value: Primitive) {
        if self.top >= STACK_SIZE {
            panic!("Stack overflow");
        }

        self.stack[self.top] = value;
        self.top += 1;
    }

    #[inline(always)]
    pub fn pop(&mut self) -> Primitive {
        if self.top == 0 {
            panic!("Stack underflow");
        }

        self.top -= 1;
        self.stack[self.top]
    }

    #[inline(always)]
    pub fn pop2(&mut self) -> (Primitive, Primitive) {
        (self.pop(), self.pop())
    }

    #[inline(always)]
    pub fn peek(&self) -> Primitive {
        self.stack[self.top - 1]
    }

    #[inline(always)]
    pub fn peek_nth(&self, nth: usize) -> Primitive {
        self.stack[self.top - 1 - nth]
    }
}

impl Index<usize> for Stack {
    type Output = Primitive;

    fn index(&self, index: usize) -> &Self::Output {
        &self.stack[index]
    }
}

impl IndexMut<usize> for Stack {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.stack[index]
    }
}

pub struct ObjectMap {
    free: usize,
    heap: Vec<Object>,
}

impl Default for ObjectMap {
    fn default() -> Self {
        Self::with_capacity(32)
    }
}

impl From<Vec<Object>> for ObjectMap {
    fn from(heap: Vec<Object>) -> Self {
        let mut free = heap.len();
        for (idx, obj) in heap.iter().enumerate() {
            if let ObjectType::Free { .. } = obj.typ {
                free = idx;
                break;
            }
        }

        Self { free, heap }
    }
}

impl ObjectMap {
    pub fn with_capacity(capacity: usize) -> Self {
        let mut heap = Vec::with_capacity(capacity);
        for i in 0..capacity {
            heap.push(Object::new(ObjectType::Free { next: i + 1 }));
        }

        Self { free: 0, heap }
    }

    pub fn allocate(&mut self, object: Object) -> usize {
        let current = self.free;
        if current >= self.heap.len() {
            self.heap
                .push(Object::new(ObjectType::Free { next: current + 1 }))
        }

        let ObjectType::Free { next } = self.heap[current].typ else {
            panic!("Allocation failed: Expected free location wasn't free");
        };

        self.heap[current] = object;
        self.free = next;

        current
    }

    pub fn get(&self, idx: usize) -> Option<&Object> {
        self.heap.get(idx)
    }

    pub fn get_mut(&mut self, idx: usize) -> Option<&mut Object> {
        self.heap.get_mut(idx)
    }
}
