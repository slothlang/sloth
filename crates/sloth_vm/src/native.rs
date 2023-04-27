use crate::value::Primitive;
use crate::VM;

pub type NativeFunctionResult = Result<Primitive, Error>;
pub type NativeFunctionInput = fn(&mut VM, &[Primitive]) -> NativeFunctionResult;

pub enum Error {
    InvalidArgument,
    Unknown(String),
}

#[allow(clippy::type_complexity)]
pub struct NativeFunction {
    pub name: &'static str,
    pub function: NativeFunctionInput,
    pub arity: u8,
    pub returns_value: bool,
    pub doc: Option<&'static str>,
}
