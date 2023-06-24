use crate::parser::ast::{AstNode, ExprKind, Stmt, StmtKind};
use crate::symtable::{Symbol, SymbolType};

#[derive(Debug, thiserror::Error)]
pub enum AnalysisError {
    #[error("Mismatched types")]
    TypeMismatch(u32),
    #[error("Unknown identifier '{1}'")]
    UnknownIdentifier(u32, String),
    #[error("Unknown error")]
    Unknown(u32),
}

impl AnalysisError {
    pub fn line(&self) -> u32 {
        match self {
            AnalysisError::TypeMismatch(line) => *line,
            AnalysisError::UnknownIdentifier(line, ..) => *line,
            AnalysisError::Unknown(line) => *line,
        }
    }
}

pub fn analyze(root: &mut Stmt) -> Result<(), AnalysisError> {
    populate_symtable(&root.as_node());
    check_usage(&root.as_node())?;

    Ok(())
}

fn populate_symtable(node: &AstNode) {
    if let AstNode::Stmt(stmt) = node {
        match &stmt.kind {
            StmtKind::DefineVariable { identifier, .. } => {
                let mut table = stmt.symtable.clone();
                table.insert(identifier.to_owned(), Symbol::new(SymbolType::Variable));
            }
            StmtKind::DefineFunction { identifier, .. } => {
                let mut table = stmt.symtable.clone();
                table.insert(identifier.to_owned(), Symbol::new(SymbolType::Function));
            }
            _ => (),
        }
    }

    for child in node.children() {
        populate_symtable(&child);
    }
}

fn check_usage(node: &AstNode) -> Result<(), AnalysisError> {
    if let AstNode::Expr(expr) = node && let ExprKind::Identifier(identifier) = &expr.kind && !expr.symtable.clone().contains(identifier) {
        return Err(AnalysisError::UnknownIdentifier(expr.line, identifier.clone()));
    }

    if let AstNode::Stmt(stmt) = node && let StmtKind::AssignVariable { identifier, .. } = &stmt.kind && !stmt.symtable.clone().contains(identifier) {
        return Err(AnalysisError::UnknownIdentifier(stmt.line, identifier.clone()));
    }

    for child in node.children() {
        check_usage(&child)?;
    }

    Ok(())
}
