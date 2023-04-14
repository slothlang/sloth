use std::io::{stdin, BufRead};

use crate::native::{self, NativeFunction, NativeFunctionResult};
use crate::value::{Object, ObjectType, Primitive};
use crate::VM;

fn write(vm: &mut VM, args: &[Primitive]) -> NativeFunctionResult {
    let Some(Primitive::Object(ptr)) = args.get(0).cloned() else {
        return Err(native::Error::InvalidArgument);
    };

    let object = vm
        .objects()
        .get(ptr as usize)
        .ok_or(native::Error::InvalidArgument)?;

    let ObjectType::String(str) = &object.typ else {
        return Err(native::Error::InvalidArgument);
    };

    print!("{str}");

    Ok(Primitive::Empty)
}

pub const WRITE_FUNCTION: NativeFunction = NativeFunction {
    name: "write",
    function: write,
    arity: 1,
    returns_value: false,
};

fn writeln(vm: &mut VM, args: &[Primitive]) -> NativeFunctionResult {
    let Some(Primitive::Object(ptr)) = args.get(0).cloned() else {
        return Err(native::Error::InvalidArgument);
    };

    let object = vm
        .objects()
        .get(ptr as usize)
        .ok_or(native::Error::InvalidArgument)?;

    let ObjectType::String(str) = &object.typ else {
        return Err(native::Error::InvalidArgument);
    };

    println!("{str}");

    Ok(Primitive::Empty)
}

pub const WRITELN_FUNCTION: NativeFunction = NativeFunction {
    name: "writeln",
    function: writeln,
    arity: 1,
    returns_value: false,
};

fn read(vm: &mut VM, _args: &[Primitive]) -> NativeFunctionResult {
    let mut line = String::new();
    stdin()
        .lock()
        .read_line(&mut line)
        .map_err(|it| native::Error::Unknown(it.to_string()))?;

    let object = Object::new(ObjectType::String(line));
    let ptr = vm.objects_mut().allocate(object);

    Ok(Primitive::Object(ptr as u32))
}

pub const READ_FUNCTION: NativeFunction = NativeFunction {
    name: "read",
    function: read,
    arity: 0,
    returns_value: true,
};
