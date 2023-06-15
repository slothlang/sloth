use std::fmt::{Error, Write};

use super::ast::{ExprKind, StmtKind};

pub struct GraphBuilder {
    i: i32,
    graph: String,
}

impl GraphBuilder {
    pub fn generate(ast: &[StmtKind]) -> Result<String, Error> {
        let mut this = Self {
            i: 0,
            graph: String::new(),
        };

        this.graph.push_str("digraph {\n");
        for stmt in ast.iter() {
            this.traverse_stmt0(stmt)?;
        }
        this.i = 0;
        for stmt in ast.iter() {
            this.traverse_stmt(stmt)?;
        }
        this.graph.push('}');

        Ok(this.graph)
    }

    fn traverse_stmt0(&mut self, stmt: &StmtKind) -> Result<(), Error> {
        self.i += 1;

        match stmt {
            StmtKind::Block(body) => {
                writeln!(&mut self.graph, "N{} [shape=box label=\"Block\"];", self.i)?;
                for stmt in body.iter() {
                    self.traverse_stmt0(stmt)?;
                }
            }
            StmtKind::ExprStmt(expr) => {
                writeln!(
                    &mut self.graph,
                    "N{} [shape=box label=\"ExprStmt\"];",
                    self.i
                )?;
                // self.traverse_expr0(expr);
            }
            StmtKind::IfStmt {
                condition,
                if_then,
                else_then,
            } => {
                writeln!(&mut self.graph, "N{} [shape=box label=\"IfStmt\"];", self.i)?;
                // self.traverse_expr0(condition);
                self.traverse_stmt0(if_then)?;
                if let Some(else_then) = else_then {
                    self.traverse_stmt0(else_then)?;
                }
            }
            StmtKind::WhileStmt { condition, body } => {
                writeln!(
                    &mut self.graph,
                    "N{} [shape=box label=\"WhileStmt\"];",
                    self.i
                )?;
                // self.traverse_expr0(condition);
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
                    self.i, identifier, typ
                )?;
                // self.traverse_expr0(value);
            }
            StmtKind::AssignVariable { identifier, value } => {
                writeln!(
                    &mut self.graph,
                    "N{} [shape=box label=\"AssignVariable\\n\\nIdentifier={}\"];",
                    self.i, identifier
                )?;
                // self.traverse_expr0(value);
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
                    self.i,
                    identifier,
                    inputs.len(),
                    output.is_some()
                )?;
                self.traverse_stmt0(body)?;
            }
            StmtKind::Return(expr) => {
                writeln!(&mut self.graph, "N{} [shape=box label=\"Return\"];", self.i)?;
                // self.traverse_expr0(expr);
            }
        }

        Ok(())
    }

    fn traverse_expr0(&mut self, expr: &ExprKind) {
        self.i += 1;
        // match expr {
        //     Expr::Grouping(_) => todo!(),
        //     Expr::Literal(_) => todo!(),
        //     Expr::Identifier(_) => todo!(),
        //     Expr::BinaryOp { op, lhs, rhs } => todo!(),
        //     Expr::UnaryOp { op, value } => todo!(),
        //     Expr::Call { callee, args } => todo!(),
        // }
    }

    fn traverse_stmt(&mut self, stmt: &StmtKind) -> Result<(), Error> {
        self.i += 1;

        match stmt {
            StmtKind::Block(_) => todo!(),
            StmtKind::ExprStmt(_) => todo!(),
            StmtKind::IfStmt {
                condition,
                if_then,
                else_then,
            } => todo!(),
            StmtKind::WhileStmt { condition, body } => todo!(),
            StmtKind::DefineVariable {
                identifier,
                value,
                typ,
            } => todo!(),
            StmtKind::AssignVariable { identifier, value } => todo!(),
            StmtKind::DefineFunction {
                identifier,
                inputs,
                output,
                body,
            } => todo!(),
            StmtKind::Return(_) => todo!(),
        }

        Ok(())
    }

    fn traverse_expr(&mut self, expr: &ExprKind) {
        //
    }
}
