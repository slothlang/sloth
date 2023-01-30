use std::fmt::Display;

use super::{Expression, Statement, Value};

impl<'a> Display for Statement<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")?;
        let value = match self {
            Statement::Val {
                identifier,
                initializer,
            } => format!("val {} {}", identifier.lexeme, initializer),
            Statement::Var {
                identifier,
                initializer,
            } => format!("var {} {}", identifier.lexeme, initializer),
            Statement::Expression { expr } => expr.to_string(),
        };
        write!(f, "{value}")?;
        write!(f, "}}")?;

        Ok(())
    }
}

impl<'a> Display for Expression<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(")?;
        let value = match self {
            Expression::Literal(value) => value.0.to_string(),
            Expression::Unary { expr, .. } => format!("+ {}", expr),
            Expression::Binary { lhs, rhs, .. } => format!("+ {lhs} {rhs}"),
        };
        write!(f, "{value}")?;
        write!(f, ")")?;

        Ok(())
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
