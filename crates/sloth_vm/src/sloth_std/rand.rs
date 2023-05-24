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
    doc: Some(
        "NativeFunction rand$gen:\n\tdesc: Returns a random number in the range `0.0 .. \
         1.0`\n\tExample: `var num = rand$gen(); # num could be any number from 0.0 to 1.0`",
    ),
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
    doc: Some(
        "NativeFunction rand$gen_range: \n\targs: min (int), max (int)\n\tdesc: Returns a random \
         numnber in the range <min> .. <max>\n\tExample: `var num = rand$gen_range(20, 76); # num \
         could be any number from 20 to 76`",
    ),
};
