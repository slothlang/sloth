use super::NATIVE_LIBRARY;
use crate::native::{self, NativeFunction, NativeFunctionResult};
use crate::value::{Object, ObjectType, Primitive};
use crate::VM;

fn get_doc(vm: &mut VM, args: &[Primitive]) -> NativeFunctionResult {
    let Some(Primitive::Object(ptr)) = args.get(0).cloned() else {
        return Err(native::Error::InvalidArgument);
    };

    let object = vm
        .objects()
        .get(ptr as usize)
        .ok_or(native::Error::InvalidArgument)?;

    let ObjectType::NativeFunction(fnc) = &object.typ else {
        return Err(native::Error::InvalidArgument);
    };

    let docs = NATIVE_LIBRARY
        .get(fnc.name)
        .ok_or(native::Error::InvalidArgument)?
        .doc
        .ok_or(native::Error::InvalidArgument)?
        .to_string();

    let object = Object::new(ObjectType::String(docs));
    let ptr = vm.objects_mut().allocate(object);

    Ok(Primitive::Object(ptr as u32))
}

pub const DOCS: NativeFunction = NativeFunction {
    name: "docs",
    function: get_doc,
    arity: 1,
    returns_value: true,
    doc: Some(
        "NativeFunction get$doc: \n\targs: name (str)\n\tdesc: Returns documentaiton on a \
         function with name <str>\n\tExample: `var doc = get$doc('wait'); # Returns the \
         documentation of the 'wait' function to doc`",
    ),
};
