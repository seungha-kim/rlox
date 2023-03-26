use std::fmt::Debug;
use std::fmt::Write;

use crate::literal::Literal;
use crate::token::TokenKind;

pub trait ExprVisitor {
    fn visit_binary(&mut self, expr: &BinaryExpr);
    fn visit_grouping(&mut self, expr: &GroupingExpr);
    fn visit_literal(&mut self, expr: &LiteralExpr);
    fn visit_unary(&mut self, expr: &UnaryExpr);
}

// TODO: https://doc.rust-lang.org/reference/procedural-macros.html
pub trait Expr {
    fn accept(&self, visitor: &mut dyn ExprVisitor);
}

pub struct BinaryExpr {
    pub left: Box<dyn Expr>,
    pub operator: TokenKind,
    pub right: Box<dyn Expr>,
}

impl Expr for BinaryExpr {
    fn accept(&self, visitor: &mut dyn ExprVisitor) {
        visitor.visit_binary(self);
    }
}

pub struct GroupingExpr(pub Box<dyn Expr>);

impl Expr for GroupingExpr {
    fn accept(&self, visitor: &mut dyn ExprVisitor) {
        visitor.visit_grouping(self);
    }
}

pub struct LiteralExpr(pub Literal);

impl Expr for LiteralExpr {
    fn accept(&self, visitor: &mut dyn ExprVisitor) {
        visitor.visit_literal(self);
    }
}

pub struct UnaryExpr {
    pub operator: TokenKind,
    pub right: Box<dyn Expr>,
}

impl Expr for UnaryExpr {
    fn accept(&self, visitor: &mut dyn ExprVisitor) {
        visitor.visit_unary(self);
    }
}
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
}

impl ExprVisitor for DepthPrinter {
    fn visit_binary(&mut self, expr: &BinaryExpr) {
        self.in_depth(|p| {
            expr.left.accept(p);
        });
        self.print_debug(&expr.operator);
        self.in_depth(|p| {
            expr.right.accept(p);
        });
    }

    fn visit_grouping(&mut self, expr: &GroupingExpr) {
        self.print_str("Group");
        self.in_depth(|p| {
            expr.0.accept(p);
        });
    }

    fn visit_literal(&mut self, expr: &LiteralExpr) {
        self.print_debug(&expr.0);
    }

    fn visit_unary(&mut self, expr: &UnaryExpr) {
        self.print_debug(&expr.operator);
        self.in_depth(|p| {
            expr.right.accept(p);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_printer() {
        let tree = BinaryExpr {
            left: Box::new(GroupingExpr(Box::new(LiteralExpr(Literal::Number(1.0))))),
            operator: TokenKind::Plus,
            right: Box::new(UnaryExpr {
                operator: TokenKind::Minus,
                right: Box::new(LiteralExpr(Literal::Number(2.0))),
            }),
        };

        let mut printer = DepthPrinter::default();
        tree.accept(&mut printer);

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
