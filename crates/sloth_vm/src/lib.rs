#![allow(dead_code)]
#![warn(
    clippy::wildcard_imports,
    clippy::string_add,
    clippy::string_add_assign,
    clippy::manual_ok_or,
    unused_lifetimes
)]

const STACK_SIZE: usize = 1024;

pub struct VM {
    stack: [u8; STACK_SIZE],
    constants: Vec<u8>,
}

#[cfg(test)]
mod tests {
    #[test]
    fn add_program() {}
}
