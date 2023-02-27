// use super::{AstVisitor, Expr, Stmt};

// pub struct AstPrettyPrinter;
// impl AstVisitor<String> for AstPrettyPrinter {
//     fn visit_stmt(&self, stmt: &Stmt) -> String {
//         match stmt {
//             Stmt::Expr(expr) => self.visit_expr(expr),
//             Stmt::Val(name, expr) => format!("(val '{}' <- {})", name,
// self.visit_expr(expr)),             Stmt::Var(name, expr) => format!("(var
// '{}' <- {})", name, self.visit_expr(expr)),         }
//     }

//     fn visit_expr(&self, expr: &Expr) -> String {
//         match expr {
//             Expr::Literal(i) => i.to_string(),
//             Expr::Add(lhs, rhs) => {
//                 let lhs = self.visit_expr(lhs);
//                 let rhs = self.visit_expr(rhs);
//                 format!("({lhs} + {rhs})")
//             }
//             Expr::Sub(lhs, rhs) => {
//                 let lhs = self.visit_expr(lhs);
//                 let rhs = self.visit_expr(rhs);
//                 format!("({lhs} - {rhs})")
//             }
//             Expr::Mul(lhs, rhs) => {
//                 let lhs = self.visit_expr(lhs);
//                 let rhs = self.visit_expr(rhs);
//                 format!("({lhs} * {rhs})")
//             }
//             Expr::Div(lhs, rhs) => {
//                 let lhs = self.visit_expr(lhs);
//                 let rhs = self.visit_expr(rhs);
//                 format!("({lhs} / {rhs})")
//             }
//         }
//     }
// }
