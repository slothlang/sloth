use std::fmt::{Error, Write};

use super::ast::{Expr, ExprKind, Stmt, StmtKind};

pub struct GraphBuilder {
    graph: String,
}

impl GraphBuilder {
    pub fn generate(ast: &[Stmt]) -> Result<String, Error> {
        let mut this = Self {
            graph: String::new(),
        };

        this.graph.push_str("digraph {\n");
        for stmt in ast.iter() {
            this.traverse_stmt0(stmt)?;
        }
        for stmt in ast.iter() {
            this.traverse_stmt(stmt)?;
        }
        this.graph.push('}');

        Ok(this.graph)
    }

    fn traverse_stmt0(&mut self, stmt: &Stmt) -> Result<(), Error> {
        match &stmt.kind {
            StmtKind::Block(body) => {
                writeln!(&mut self.graph, "N{} [shape=box label=\"Block\"];", stmt.id)?;
                for stmt in body.iter() {
                    self.traverse_stmt0(stmt)?;
                }
            }
            StmtKind::ExprStmt(expr) => {
                writeln!(
                    &mut self.graph,
                    "N{} [shape=box label=\"ExprStmt\"];",
                    stmt.id,
                )?;
                self.traverse_expr0(expr)?;
            }
            StmtKind::IfStmt {
                condition,
                if_then,
                else_then,
            } => {
                writeln!(
                    &mut self.graph,
                    "N{} [shape=box label=\"IfStmt\"];",
                    stmt.id
                )?;
                self.traverse_expr0(condition)?;
                self.traverse_stmt0(if_then)?;
                if let Some(else_then) = else_then {
                    self.traverse_stmt0(else_then)?;
                }
            }
            StmtKind::WhileStmt { condition, body } => {
                writeln!(
                    &mut self.graph,
                    "N{} [shape=box label=\"WhileStmt\"];",
                    stmt.id
                )?;
                self.traverse_expr0(condition)?;
                self.traverse_stmt0(body)?;
            }
            StmtKind::DefineVariable {
                identifier,
                value,
                typ,
            } => {
                writeln!(
                    &mut self.graph,
                    "N{} [shape=box label=\"DefineVariable\\n\\nIdentifier={}\\nType={}\"];",
                    stmt.id, identifier, typ
                )?;
                self.traverse_expr0(value)?;
            }
            StmtKind::AssignVariable { identifier, value } => {
                writeln!(
                    &mut self.graph,
                    "N{} [shape=box label=\"AssignVariable\\n\\nIdentifier={}\"];",
                    stmt.id, identifier
                )?;
                self.traverse_expr0(value)?;
            }
            StmtKind::DefineFunction {
                identifier,
                inputs,
                output,
                body,
            } => {
                writeln!(
                    &mut self.graph,
                    "N{} [shape=box \
                     label=\"DefineFunction\\n\\nIdentifier={}\\nInputs={}\\nOutput={}\"];",
                    stmt.id,
                    identifier,
                    inputs.len(),
                    output.is_some()
                )?;
                self.traverse_stmt0(body)?;
            }
            StmtKind::Return(expr) => {
                writeln!(
                    &mut self.graph,
                    "N{} [shape=box label=\"Return\"];",
                    stmt.id
                )?;
                self.traverse_expr0(expr)?;
            }
        }

        Ok(())
    }

    fn traverse_expr0(&mut self, expr: &Expr) -> Result<(), Error> {
        match &expr.kind {
            ExprKind::Grouping(child) => {
                writeln!(
                    &mut self.graph,
                    "N{} [shape=circle label=\"Grouping\"];",
                    expr.id
                )?;
                self.traverse_expr0(child)?;
            }
            ExprKind::Literal(literal) => {
                writeln!(
                    &mut self.graph,
                    "N{} [shape=diamond label=\"Literal\\n\\nValue={}\"];",
                    expr.id, literal
                )?;
            }
            ExprKind::Identifier(identifier) => {
                writeln!(
                    &mut self.graph,
                    "N{} [shape=diamond label=\"Identifier\\n\\nIdentifier={}\"];",
                    expr.id, identifier
                )?;
            }
            ExprKind::BinaryOp { op, lhs, rhs } => {
                writeln!(
                    &mut self.graph,
                    "N{} [shape=circle label=\"{}\"];",
                    expr.id, op
                )?;
                self.traverse_expr0(lhs)?;
                self.traverse_expr0(rhs)?;
            }
            ExprKind::UnaryOp { op, value } => {
                writeln!(
                    &mut self.graph,
                    "N{} [shape=circle label=\"Unary {}\"];",
                    expr.id, op
                )?;
                self.traverse_expr0(value)?;
            }
            ExprKind::Call { callee, args } => {
                writeln!(
                    &mut self.graph,
                    "N{} [shape=circle label=\"Function Call\"];",
                    expr.id
                )?;
                self.traverse_expr0(callee)?;
                for arg in args {
                    self.traverse_expr0(arg)?;
                }
            }
        }

        Ok(())
    }

    fn traverse_stmt(&mut self, stmt: &Stmt) -> Result<(), Error> {
        match &stmt.kind {
            StmtKind::Block(children) => {
                for child in children {
                    writeln!(&mut self.graph, "N{} -> N{};", stmt.id, child.id)?;
                    self.traverse_stmt(child)?;
                }
            }
            StmtKind::ExprStmt(expr) => {
                writeln!(&mut self.graph, "N{} -> N{};", stmt.id, expr.id)?;
                self.traverse_expr(expr)?;
            }
            StmtKind::IfStmt {
                if_then, else_then, ..
            } => {
                writeln!(
                    &mut self.graph,
                    "N{} -> N{} [label = \"If Then\"];",
                    stmt.id, if_then.id
                )?;
                self.traverse_stmt(if_then)?;
                if let Some(else_then) = else_then {
                    writeln!(
                        &mut self.graph,
                        "N{} -> N{} [label = \"Else Then\"];",
                        stmt.id, else_then.id
                    )?;
                    self.traverse_stmt(else_then)?;
                }
            }
            StmtKind::WhileStmt { condition, body } => {
                writeln!(
                    &mut self.graph,
                    "N{} -> N{} [label = \"Condition\"];",
                    stmt.id, condition.id
                )?;
                writeln!(
                    &mut self.graph,
                    "N{} -> N{} [label = \"Body\"];",
                    stmt.id, body.id
                )?;
                self.traverse_expr(condition)?;
                self.traverse_stmt(body)?;
            }
            StmtKind::DefineVariable { value, .. } => {
                writeln!(&mut self.graph, "N{} -> N{};", stmt.id, value.id)?;
                self.traverse_expr(value)?;
            }
            StmtKind::AssignVariable { value, .. } => {
                writeln!(&mut self.graph, "N{} -> N{};", stmt.id, value.id)?;
                self.traverse_expr(value)?;
            }
            StmtKind::DefineFunction { body, .. } => {
                writeln!(
                    &mut self.graph,
                    "N{} -> N{} [label = \"Body\"];",
                    stmt.id, body.id
                )?;
                self.traverse_stmt(body)?;
            }
            StmtKind::Return(_) => (),
        }

        Ok(())
    }

    fn traverse_expr(&mut self, expr: &Expr) -> Result<(), Error> {
        match &expr.kind {
            ExprKind::Grouping(children) => {
                writeln!(&mut self.graph, "N{} -> N{};", expr.id, children.id)?;
                self.traverse_expr(children)?;
            }
            ExprKind::BinaryOp { lhs, rhs, .. } => {
                writeln!(&mut self.graph, "N{} -> N{} [label=lhs];", expr.id, lhs.id)?;
                writeln!(&mut self.graph, "N{} -> N{} [label=rhs];", expr.id, rhs.id)?;
                self.traverse_expr(lhs)?;
                self.traverse_expr(rhs)?;
            }
            ExprKind::UnaryOp { value, .. } => {
                writeln!(&mut self.graph, "N{} -> N{};", expr.id, value.id)?;
                self.traverse_expr(value)?;
            }
            ExprKind::Call { callee, args } => {
                writeln!(&mut self.graph, "N{} -> N{};", expr.id, callee.id)?;
                self.traverse_expr(callee)?;
                for arg in args {
                    writeln!(&mut self.graph, "N{} -> N{};", expr.id, arg.id)?;
                    self.traverse_expr(arg)?;
                }
            }
            _ => (),
        }

        Ok(())
    }
}
