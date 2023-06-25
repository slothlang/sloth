pub mod setup;

use crate::parser::ast::{AstNode, ExprKind, Stmt, StmtKind};

#[derive(Debug, thiserror::Error)]
pub enum AnalysisError {
    #[error("Mismatched types")]
    TypeMismatch(u32),
    #[error("Unknown identifier '{1}'")]
    UnknownIdentifier(u32, String),
    #[error("Unknown error '{1}'")]
    Unknown(u32, &'static str),
}

impl AnalysisError {
    pub fn line(&self) -> u32 {
        match self {
            AnalysisError::TypeMismatch(line) => *line,
            AnalysisError::UnknownIdentifier(line, ..) => *line,
            AnalysisError::Unknown(line, ..) => *line,
        }
    }
}

pub fn analyze(root: &mut Stmt) -> Result<(), AnalysisError> {
    setup::populate_symtable(&root.as_node())?;
    setup::propagate_types_stmt(root)?;

    check_usage(&root.as_node())?;

    Ok(())
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
