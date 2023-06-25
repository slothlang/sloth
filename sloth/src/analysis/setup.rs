use super::AnalysisError;
use crate::parser::ast::{
    AstNode, Expr, ExprKind, Function, FunctionInput, FunctionKind, Literal, Stmt, StmtKind,
};
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

pub(super) fn propagate_types_stmt(node: &mut Stmt) -> Result<(), AnalysisError> {
    match &mut node.kind {
        StmtKind::Block(children) => {
            for child in children {
                propagate_types_stmt(child)?;
            }
        }
        StmtKind::ExprStmt(expr) => {
            propagate_types(expr)?;
        }
        StmtKind::IfStmt {
            condition,
            if_then,
            else_then,
        } => {
            propagate_types(condition)?;
            propagate_types_stmt(if_then)?;
            if let Some(else_then) = else_then {
                propagate_types_stmt(else_then)?;
            }
        }
        StmtKind::WhileStmt { condition, body } => {
            propagate_types(condition)?;
            propagate_types_stmt(body)?;
        }
        StmtKind::DefineVariable { value, .. } => {
            propagate_types(value)?;
        }
        StmtKind::AssignVariable { value, .. } => {
            propagate_types(value)?;
        }
        StmtKind::DefineFunction(function) => {
            if let FunctionKind::Normal { body } = &mut function.kind {
                propagate_types_stmt(body)?;
            }
        }
        StmtKind::Return(expr) => {
            propagate_types(expr)?;
        }
    }

    Ok(())
}

pub(super) fn propagate_types(node: &mut Expr) -> Result<(), AnalysisError> {
    let typ = match &mut node.kind {
        ExprKind::Grouping(child) => {
            propagate_types(child)?;
            child
                .typ
                .clone()
                .ok_or(AnalysisError::Unknown(node.line, "owo choco"))?
        }
        ExprKind::Literal(lit) => match lit {
            Literal::Integer(_) => Type::Integer,
            Literal::Float(_) => Type::Float,
            Literal::Boolean(_) => Type::Boolean,
            _ => todo!(),
        },
        ExprKind::Identifier(identifier) => {
            let table = node.symtable.clone();
            table
                .get_value(identifier)
                .ok_or(AnalysisError::UnknownIdentifier(
                    node.line,
                    identifier.to_owned(),
                ))?
        }
        ExprKind::BinaryOp { lhs, rhs, .. } => {
            // Propagating the types to the children
            propagate_types(lhs)?;
            propagate_types(rhs)?;

            if lhs.typ != rhs.typ {
                return Err(AnalysisError::TypeMismatch(node.line));
            }

            lhs.typ
                .clone()
                .ok_or(AnalysisError::Unknown(node.line, "owo?? choco???"))?
        }
        ExprKind::UnaryOp { value, .. } => {
            propagate_types(value)?;

            value.typ.clone().ok_or(AnalysisError::Unknown(
                node.line,
                "YOU'RE WRONG... SULFURIC ACID!",
            ))?
        }
        ExprKind::Call { callee, args } => {
            propagate_types(callee)?;
            for arg in args {
                propagate_types(arg)?;
            }

            let Some(Type::Function { ref output, .. }) = callee.typ else {
                return Err(AnalysisError::TypeMismatch(node.line));
            };

            *output.clone()
        }
    };

    node.typ = Some(typ);

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::analysis::setup::propagate_types;
    use crate::parser::ast::{BinaryOp, Expr, ExprKind, Literal};
    use crate::symtable::{Symbol, SymbolTable, Type};

    #[test]
    fn haiiiiiuwu() {
        let mut table = SymbolTable::new();
        table.insert("poggo".to_owned(), Symbol::Value(Type::Integer));
        table.insert("poggu".to_owned(), Symbol::Value(Type::Float));

        let mut x = Expr::new(
            0,
            0,
            ExprKind::BinaryOp {
                op: BinaryOp::Add,
                lhs: Box::new(Expr::new(1, 0, Literal::Float(1.).into(), table.clone())),
                rhs: Box::new(Expr::new(
                    2,
                    0,
                    ExprKind::Identifier("poggu".to_owned()),
                    table.clone(),
                )),
            },
            table,
        );

        propagate_types(&mut x).expect("oh noes something went fucky wucky >~<");

        println!("{x:#?}");
        panic!()
    }
}
