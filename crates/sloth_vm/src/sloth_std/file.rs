use std::fs;

use crate::native::{self, NativeFunction, NativeFunctionResult};
use crate::value::{Object, ObjectType, Primitive};
use crate::VM;

fn file_read(vm: &mut VM, args: &[Primitive]) -> NativeFunctionResult {
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

    let contents = fs::read_to_string(str).expect("IO Error: Failed to read file!");

    let object = Object::new(ObjectType::String(contents));
    let ptr = vm.objects_mut().allocate(object);

    Ok(Primitive::Object(ptr as u32))
}

pub const FILE_READ: NativeFunction = NativeFunction {
    name: "file$read",
    function: file_read,
    arity: 1,
    returns_value: true,
    doc: None,
};

fn file_write(vm: &mut VM, args: &[Primitive]) -> NativeFunctionResult {
    let Some(Primitive::Object(path_ptr)) = args.get(0).cloned() else {
        return Err(native::Error::InvalidArgument);
    };

    let path_object = vm
        .objects()
        .get(path_ptr as usize)
        .ok_or(native::Error::InvalidArgument)?;

    let ObjectType::String(path) = &path_object.typ else {
        return Err(native::Error::InvalidArgument);
    };

    let Some(Primitive::Object(content_ptr)) = args.get(1).cloned() else {
        return Err(native::Error::InvalidArgument);
    };

    let content_object = vm
        .objects()
        .get(content_ptr as usize)
        .ok_or(native::Error::InvalidArgument)?;

    let ObjectType::String(content) = &content_object.typ else {
        return Err(native::Error::InvalidArgument);
    };

    let _ = fs::write(path, content);

    Ok(Primitive::Empty)
}

pub const FILE_WRITE: NativeFunction = NativeFunction {
    name: "file$write",
    function: file_write,
    arity: 2,
    returns_value: false,
    doc: None,
};
