use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::parse::Parse;
use syn::punctuated::Punctuated;
use syn::{bracketed, parse_macro_input, LitInt, LitStr, Token};

// TODO: Rename args to operands?

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

    if args.len() > 0 {
        quote! {
            #[doc = #description]
            #name ( #( #args ),*  ) = #opcode
        }
    } else {
        quote! {
            #[doc = #description]
            #name = #opcode
        }
    }
}

fn into_bytecode_parser(instruction: &DslInstructionInput) -> TokenStream {
    let DslInstructionInput {
        opcode,
        name,
        args,
        description: _,
    } = instruction;

    if args.is_empty() {
        return quote! {
            #opcode => Self :: #name
        };
    }

    let mut arg_params = Vec::new();
    for arg in args {
        let size = match arg.to_string().as_str() {
            "u128" => 128,
            "u64" => 64,
            "u32" => 32,
            "u16" => 16,
            "u8" => 8,
            typ => panic!("Unsupported instruction arg type '{typ}'"),
        } as usize;

        let bytes = size / 8;

        let mut chunks = Vec::new();
        for byte in 0..bytes {
            let shift_amount = size - (byte + 1) * bytes;
            chunks.push(quote! {
                ((chunk.code[*offset + #byte] as #arg) << #shift_amount)
            });
        }

        arg_params.push(quote! {
            let value = #( #chunks )+* ;
            *offset += #bytes ;
            value
        });
    }

    quote! {
        #opcode => {
            Self :: #name (
                #( { #arg_params } ),*
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
    quote! {
        #[repr(u8)]
        #[derive(Clone, Debug, Eq, PartialEq)]
        enum #enum_name {
            #( #enum_fields ),*
        }

        impl #enum_name {
            fn disassemble(chunk: &Chunk, offset: &mut usize) -> #enum_name {
                let opcode = chunk.code[*offset];
                *offset += 1;

                let instruction = match opcode {
                    #( #bytecode_parsers , )*
                    _ => panic!("Unknown bytecode encountered"),
                };

                instruction
            }

            fn assemble(chunk: &mut Chunk) {
                //
            }
        }
    }
    .into()
}
