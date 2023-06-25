use super::AnalysisError;
use crate::parser::ast::{AstNode, Function, FunctionInput, FunctionKind, StmtKind};
use crate::symtable::{Symbol, SymbolTable, Type};

pub(super) fn populate_symtable(node: &AstNode) -> Result<(), AnalysisError> {
    if let AstNode::Stmt(stmt) = node {
        let mut table = stmt.symtable.clone();

        match &stmt.kind {
            StmtKind::DefineVariable {
                identifier, typ, ..
            } => {
                // When a variable is defined add it to the symbol table of the current scope.
                let symbol = build_value_symbol(&table, typ)?;
                table.insert(identifier.to_owned(), symbol);
            }
            StmtKind::DefineFunction(Function {
                identifier,
                inputs,
                output,
                kind,
            }) => {
                // When a function is defined add the function to the symbol
                // table of the current scope, and add the inputs to the child
                // (body) scope.
                let function_symbol = build_function_symbol(&table, inputs, output.as_deref())?;
                table.insert(identifier.to_owned(), function_symbol);

                if let FunctionKind::Normal { body } = kind {
                    let mut body_table = body.symtable.clone();

                    for input in inputs {
                        let symbol = build_value_symbol(&body_table, &input.typ)?;
                        body_table.insert(input.identifier.to_owned(), symbol);
                    }
                }
            }
            _ => (),
        }
    }

    for child in node.children() {
        populate_symtable(&child)?;
    }

    Ok(())
}

fn build_value_symbol(table: &SymbolTable, typ: &str) -> Result<Symbol, AnalysisError> {
    let typ = table
        .get_type(typ)
        .ok_or(AnalysisError::UnknownIdentifier(0, typ.to_owned()))?;

    Ok(Symbol::Value(typ))
}

fn build_function_symbol(
    table: &SymbolTable,
    inputs: &[FunctionInput],
    output: Option<&str>,
) -> Result<Symbol, AnalysisError> {
    let inputs = inputs
        .iter()
        .map(|it| table.get_type(&it.typ))
        .collect::<Option<Vec<_>>>()
        .ok_or(AnalysisError::UnknownIdentifier(0, "0xOwO".to_owned()))?;

    let output = output
        .map(|it| table.get_type(it))
        .unwrap_or(Some(Type::Void))
        .ok_or(AnalysisError::UnknownIdentifier(0, "0xUwU".to_owned()))?;

    Ok(Symbol::Value(Type::Function {
        inputs,
        output: output.into(),
    }))
}
