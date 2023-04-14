use rand::Rng;

use crate::native::{self, NativeFunction, NativeFunctionResult};
use crate::value::Primitive;
use crate::value::Primitive::{Float, Integer};
use crate::VM;

fn gen(_vm: &mut VM, _args: &[Primitive]) -> NativeFunctionResult {
    let value = rand::thread_rng().gen_range(0.0..1.0);

    Ok(Float(value))
}

pub const GEN_FUNCTION: NativeFunction = NativeFunction {
    name: "rand$gen",
    function: gen,
    arity: 0,
    returns_value: true,
};

fn gen_range(_vm: &mut VM, args: &[Primitive]) -> NativeFunctionResult {
    let min = args.get(0).cloned();
    let max = args.get(1).cloned();

    let (Some(Integer(min)), Some(Integer(max))) = (min, max) else {
        return Err(native::Error::InvalidArgument);
    };

    let value = rand::thread_rng().gen_range(min..max);

    Ok(Integer(value))
}

pub const GEN_RANGE_FUNCTION: NativeFunction = NativeFunction {
    name: "rand$gen_range",
    function: gen_range,
    arity: 2,
    returns_value: true,
};
