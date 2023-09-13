use std::fmt::{Error, Write};

use super::ast::{Expr, ExprKind, Function, FunctionKind, Stmt, StmtKind};

pub struct GraphBuilder {
    graph: String,
}

impl GraphBuilder {
    pub fn generate(source: Option<&str>, ast: &Stmt) -> Result<String, Error> {
        let mut this = Self {
            graph: String::new(),
        };

        this.graph.push_str("digraph {\n");

        if let Some(source) = source {
            let source = source
                .replace('\"', "\\\"")
                .replace("\\n", "\\\\n")
                .replace('\n', "\\l");

            this.graph.push_str(&format!("label = \"{source}\";"));
            this.graph.push_str("labeljust = l; labelloc = t;");
        }

        this.traverse_stmt0(ast)?;
        this.traverse_stmt(ast)?;
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
            StmtKind::DefineStruct {
                identifier: _,
                properties: _,
            } => {
                todo!();
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
            StmtKind::ForStmt {
                iterator,
                identifier: _,
                body,
            } => {
                writeln!(
                    &mut self.graph,
                    "N{} [shape=box label=\"ForStmt\"];",
                    stmt.id
                )?;
                self.traverse_expr0(iterator)?;
                self.traverse_stmt0(body)?;
            }
            StmtKind::DefineValue {
                identifier,
                value,
                typ,
            } => {
                writeln!(
                    &mut self.graph,
                    "N{} [shape=box label=\"DefineValue\\n\\nIdentifier={}\\lType={}\\l\"];",
                    stmt.id,
                    identifier,
                    typ.clone().unwrap()
                )?;
                self.traverse_expr0(value)?;
            }
            StmtKind::DefineVariable {
                identifier,
                value,
                typ,
            } => {
                writeln!(
                    &mut self.graph,
                    "N{} [shape=box label=\"DefineVariable\\n\\nIdentifier={}\\lType={}\\l\"];",
                    stmt.id,
                    identifier,
                    typ.clone().unwrap()
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
            StmtKind::DefineFunction(Function {
                identifier,
                inputs,
                output,
                kind,
            }) => {
                writeln!(
                    &mut self.graph,
                    "N{} [shape=box \
                     label=\"DefineFunction\\n\\nIdentifier={}\\lInputs={}\\lOutput={}\\lKind={}\\\
                     l\"];",
                    stmt.id,
                    identifier,
                    inputs.len(),
                    output.is_some(),
                    match kind {
                        FunctionKind::Normal { .. } => "Normal",
                        FunctionKind::Foreign => "Foreign",
                    }
                )?;

                if let FunctionKind::Normal { body } = kind {
                    self.traverse_stmt0(body)?;
                }
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
                    "N{} [shape=box style=rounded label=\"Grouping\"];",
                    expr.id
                )?;
                self.traverse_expr0(child)?;
            }
            ExprKind::Literal(literal) => {
                writeln!(
                    &mut self.graph,
                    "N{} [shape=box style=\"filled,rounded\" label=\"{}\"];",
                    expr.id, literal
                )?;
            }
            ExprKind::Identifier(identifier) => {
                writeln!(
                    &mut self.graph,
                    "N{} [shape=box style=\"filled,rounded\" label=\"{}\"];",
                    expr.id, identifier
                )?;
            }
            ExprKind::BinaryOp { op, lhs, rhs } => {
                writeln!(
                    &mut self.graph,
                    "N{} [shape=box style=rounded label=\"{}\"];",
                    expr.id, op
                )?;
                self.traverse_expr0(lhs)?;
                self.traverse_expr0(rhs)?;
            }
            ExprKind::UnaryOp { op, value } => {
                writeln!(
                    &mut self.graph,
                    "N{} [shape=box style=rounded label=\"{}\"];",
                    expr.id, op
                )?;
                self.traverse_expr0(value)?;
            }
            ExprKind::Call { callee, args } => {
                writeln!(
                    &mut self.graph,
                    "N{} [shape=box style=rounded label=\"Function Call\"];",
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
            StmtKind::DefineStruct {
                identifier: _,
                properties: _,
            } => todo!(),
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
                condition,
                if_then,
                else_then,
                ..
            } => {
                writeln!(
                    &mut self.graph,
                    "N{} -> N{} [label = \"Condition\"];",
                    stmt.id, condition.id
                )?;
                self.traverse_expr(condition)?;
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
            StmtKind::ForStmt {
                iterator,
                identifier: _,
                body,
            } => {
                writeln!(
                    &mut self.graph,
                    "N{} -> N{} [label = \"Iterator\"];",
                    stmt.id, iterator.id
                )?;
                writeln!(
                    &mut self.graph,
                    "N{} -> N{} [label = \"Body\"];",
                    stmt.id, body.id
                )?;
                self.traverse_expr(iterator)?;
                self.traverse_stmt(body)?;
            }
            StmtKind::DefineVariable { value, .. } => {
                writeln!(&mut self.graph, "N{} -> N{};", stmt.id, value.id)?;
                self.traverse_expr(value)?;
            }
            StmtKind::DefineValue { value, .. } => {
                writeln!(&mut self.graph, "N{} -> N{};", stmt.id, value.id)?;
                self.traverse_expr(value)?;
            }
            StmtKind::AssignVariable { value, .. } => {
                writeln!(&mut self.graph, "N{} -> N{};", stmt.id, value.id)?;
                self.traverse_expr(value)?;
            }
            StmtKind::DefineFunction(Function { kind, .. }) => {
                if let FunctionKind::Normal { body } = kind {
                    writeln!(
                        &mut self.graph,
                        "N{} -> N{} [label = \"Body\"];",
                        stmt.id, body.id
                    )?;

                    self.traverse_stmt(body)?;
                }
            }
            StmtKind::Return(value) => {
                writeln!(&mut self.graph, "N{} -> N{};", stmt.id, value.id)?;
                self.traverse_expr(value)?;
            }
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
                writeln!(
                    &mut self.graph,
                    "N{} -> N{} [label=callee];",
                    expr.id, callee.id
                )?;
                self.traverse_expr(callee)?;
                for arg in args {
                    writeln!(&mut self.graph, "N{} -> N{} [label=arg];", expr.id, arg.id)?;
                    self.traverse_expr(arg)?;
                }
            }
            _ => (),
        }

        Ok(())
    }
}
