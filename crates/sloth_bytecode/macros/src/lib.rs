use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::parse::Parse;
use syn::punctuated::Punctuated;
use syn::{bracketed, parse_macro_input, LitInt, LitStr, Token};

struct DslInstructionInput {
    opcode: LitInt,
    name: Ident,
    args: Punctuated<Ident, Token![,]>,
    description: LitStr,
}

impl Parse for DslInstructionInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let args_content;
        Ok(Self {
            opcode: input.parse()?,
            name: input.parse()?,
            args: {
                bracketed!(args_content in input);
                args_content.parse_terminated(Ident::parse, Token![,])?
            },
            description: input.parse()?,
        })
    }
}

struct DslInstructionsInput {
    name: Ident,
    instructions: Punctuated<DslInstructionInput, Token![,]>,
}

impl Parse for DslInstructionsInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            name: input.parse()?,
            instructions: {
                input.parse::<Token![;]>()?;
                input.parse_terminated(DslInstructionInput::parse, Token![,])?
            },
        })
    }
}

fn into_enum_field(instruction: &DslInstructionInput) -> TokenStream {
    let DslInstructionInput {
        opcode,
        name,
        args,
        description,
    } = instruction;

    let args = args.iter();

    quote! {
        #[doc = #description]
        #name ( #( #args ),*  ) = #opcode
    }
}

fn into_bytecode_parser(instruction: &DslInstructionInput) -> TokenStream {
    let DslInstructionInput {
        opcode,
        name,
        args,
        description: _,
    } = instruction;

    let args = args.iter().map(|arg| {
        let read_ident = format_ident!("read_{}", arg);

        let _chunk_codes = arg;

        quote! {
            {
                let a: #arg = (chunk.code[*offset] << 56) + (chunk)
                cursor . #read_ident ::<byteorder::LittleEndian>().unwrap()
            }
        }
    });

    quote! {
        #opcode => {
            Self:: #name (
                #( #args ),*
            )
        }
    }
}

#[proc_macro]
pub fn instructions(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DslInstructionsInput);

    // Getting values to construct the enum
    let enum_name = input.name;
    let enum_fields = input
        .instructions
        .iter()
        .map(into_enum_field)
        .collect::<Vec<_>>();

    // Getting the values to parse bytecode
    let bytecode_parsers = input
        .instructions
        .iter()
        .map(into_bytecode_parser)
        .collect::<Vec<_>>();

    // Building out the expanded code
    let expanded = quote! {
        #[repr(u8)]
        #[derive(Clone, Debug)]
        enum #enum_name {
            #( #enum_fields ),*
        }

        impl #enum_name {
            fn disassemble(chunk: &Chunk, offset: &mut usize) -> #enum_name {
                let opcode = chunk.code[*offset];
                *offset += 1;

                let instruction = match opcode {
                    #( #bytecode_parsers ),*
                    _ => panic!("Unknown bytecode encountered"),
                };

                instruction
            }

            fn assemble(chunk: &mut Chunk) {
                //
            }
        }

        // impl #enum_name {
        //     fn from_bytecode(cursor: &mut Cursor<Vec<u8>>) -> Self {
        //         let bytecode = cursor.read_u8().unwrap();
        //
        //         let instruction = match bytecode {
        //             #( #bytecode_parsers ),*
        //             _ => panic!("Unknown bytecode encountered"),
        //         };
        //
        //         instruction
        //     }
        // }
    };

    // Returning the proc_macro version of TokenStream
    expanded.into()
}
