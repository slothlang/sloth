use crate::native::{self, NativeFunction, NativeFunctionResult};
use crate::value::Primitive;
use crate::value::Primitive::Integer;
use crate::VM;

pub const TERM_CLEAR: NativeFunction = NativeFunction {
    name: "term$clear",
    function: |_vm, _args| {
        print!("\x1b[2J\x1b[H");
        Ok(Primitive::Empty)
    },
    arity: 0,
    returns_value: false,
    doc: Some(
        "NativeFunction term_clear: \n\tdesc: Clears the terminal\n\tExample: `term_clear(); # \
         Clears the terminal`",
    ),
};

fn term_setpos(_vm: &mut VM, args: &[Primitive]) -> NativeFunctionResult {
    let x = args.get(0).cloned();
    let y = args.get(1).cloned();

    let (Some(Integer(x)), Some(Integer(y))) = (x, y) else {
        return Err(native::Error::InvalidArgument);
    };
    print!("\x1b[{x};{y}H");
    Ok(Primitive::Empty)
}

pub const TERM_SETPOS: NativeFunction = NativeFunction {
    name: "term$setpos",
    function: term_setpos,
    arity: 2,
    returns_value: false,
    doc: Some(
        "NativeFunction term_setpos: \n\targs: x (int), y (int)\n\tdesc: Sets the cursors \
         position to (<x>, <y>)\n\tExample: `term_setpos(5, 17); # Sets the position of the \
         cursor to (5, 17)`",
    ),
};
