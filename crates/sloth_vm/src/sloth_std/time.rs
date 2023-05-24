use std::{thread, time};

use crate::native::{self, NativeFunction, NativeFunctionResult};
use crate::value::Primitive;
use crate::value::Primitive::Integer;
use crate::VM;

fn wait(_vm: &mut VM, args: &[Primitive]) -> NativeFunctionResult {
    let sec = args.get(0).cloned();

    let Some(Integer(sec)) = sec else {
        return Err(native::Error::InvalidArgument);
    };

    thread::sleep(time::Duration::from_secs(sec.try_into().unwrap()));

    Ok(Primitive::Empty)
}

pub const WAIT: NativeFunction = NativeFunction {
    name: "wait",
    function: wait,
    arity: 1,
    returns_value: false,
    doc: Some(
        "NativeFunction wait: \n\targs: sec (int)\n\tdesc: Waits for <sec> seconds.\n\tExample: \
         `wait(10); # Waits 10 seconds`",
    ),
};
