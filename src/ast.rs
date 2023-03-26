use std::fmt::Debug;
use std::fmt::Write;

use crate::token::TokenKind;
use crate::value::Value;

pub enum Expr {
    BinaryExpr {
        left: Box<Expr>,
        operator: TokenKind,
        right: Box<Expr>,
    },
    GroupingExpr(Box<Expr>),
    LiteralExpr(Value),
    UnaryExpr {
        operator: TokenKind,
        right: Box<Expr>,
    },
}

pub use Expr::*;

#[derive(Default)]
pub struct DepthPrinter {
    buf: String,
    depth: usize,
}

impl DepthPrinter {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn print(&self) {
        println!("{}", self.buf);
    }

    fn inc(&mut self) {
        self.depth += 1;
    }

    fn desc(&mut self) {
        self.depth -= 1;
    }

    fn print_depth(&mut self) {
        let depth_str = "  ".repeat(self.depth);
        write!(&mut self.buf, "{}", &depth_str).unwrap();
    }

    fn print_debug(&mut self, token: &dyn Debug) {
        self.print_depth();
        writeln!(&mut self.buf, "{:?}", token).unwrap();
    }

    fn print_str(&mut self, s: &str) {
        self.print_depth();
        writeln!(&mut self.buf, "{}", s).unwrap();
    }

    fn in_depth(&mut self, f: impl Fn(&mut Self)) {
        self.inc();
        f(self);
        self.desc();
    }

    pub fn visit(&mut self, expr: &Expr) {
        match expr {
            BinaryExpr {
                left,
                operator,
                right,
            } => {
                self.in_depth(|p| {
                    p.visit(left);
                });
                self.print_debug(operator);
                self.in_depth(|p| {
                    p.visit(right);
                });
            }
            GroupingExpr(expr) => {
                self.print_str("Group");
                self.in_depth(|p| {
                    p.visit(expr);
                });
            }
            LiteralExpr(lit) => {
                self.print_debug(lit);
            }
            UnaryExpr { operator, right } => {
                self.print_debug(operator);
                self.in_depth(|p| {
                    p.visit(right);
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_printer() {
        use Expr::*;
        let tree = BinaryExpr {
            left: Box::new(GroupingExpr(Box::new(LiteralExpr(Value::Number(1.0))))),
            operator: TokenKind::Plus,
            right: Box::new(UnaryExpr {
                operator: TokenKind::Minus,
                right: Box::new(LiteralExpr(Value::Number(2.0))),
            }),
        };

        let mut printer = DepthPrinter::default();
        printer.visit(&tree);

        #[rustfmt::skip]
let expected =
"  Group
    Number(1.0)
Plus
  Minus
    Number(2.0)
";
        assert_eq!(printer.buf, expected);
    }
}
