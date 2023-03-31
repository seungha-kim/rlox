use anyhow::bail;
use rlox_syntax::{Expr, Statement};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

enum VariableState {
    Declared,
    Initialized,
}

pub type ScopePtr = Rc<RefCell<Scope>>;

pub struct Scope {
    parent: Option<ScopePtr>,
    variables: HashMap<String, VariableState>,
}

impl Scope {
    pub fn new_ptr(parent: Option<ScopePtr>) -> ScopePtr {
        Rc::new(RefCell::new(Self::new(parent)))
    }

    fn new(parent: Option<ScopePtr>) -> Self {
        Self {
            parent,
            variables: HashMap::new(),
        }
    }

    fn resolve(&self, name: &str) -> Option<usize> {
        if let Some(&VariableState::Initialized) = self.variables.get(name) {
            Some(0)
        } else if let Some(parent) = &self.parent {
            parent.borrow_mut().resolve(name).map(|n| n + 1)
        } else {
            None
        }
    }
}

pub struct ResolvedStatement(pub Statement);

pub struct Resolver;

impl Resolver {
    pub fn resolve_statement(
        &mut self,
        scope: &ScopePtr,
        statement: &mut Statement,
    ) -> anyhow::Result<()> {
        match statement {
            Statement::Expression(stmt) => {
                self.resolve_expression(scope, &mut stmt.expr)?;
            }
            Statement::Print(stmt) => {
                self.resolve_expression(scope, &mut stmt.expr)?;
            }
            Statement::VariableDecl(stmt) => {
                if scope.borrow().variables.contains_key(&stmt.name) {
                    bail!(
                        "Already a variable with this name in this scope: {}",
                        stmt.name
                    )
                }
                scope
                    .borrow_mut()
                    .variables
                    .insert(stmt.name.clone(), VariableState::Declared);
                if let Some(expr) = &mut stmt.expr {
                    self.resolve_expression(scope, expr)?;
                }
                scope
                    .borrow_mut()
                    .variables
                    .insert(stmt.name.clone(), VariableState::Initialized);
            }
            Statement::Block(stmt) => {
                let mut scope = Scope::new_ptr(Some(scope.clone()));
                for s in &mut stmt.statements {
                    self.resolve_statement(&mut scope, s)?;
                }
            }
            Statement::If(stmt) => {
                self.resolve_expression(scope, &mut stmt.condition)?;
                self.resolve_statement(scope, &mut stmt.then_branch)?;
                if let Some(else_branch) = &mut stmt.else_branch {
                    self.resolve_statement(scope, else_branch)?;
                }
            }
            Statement::While(stmt) => {
                self.resolve_expression(scope, &mut stmt.condition)?;
                self.resolve_statement(scope, &mut stmt.body)?;
            }
            Statement::Function(stmt) => {
                // TODO: scope 관련 처리가 interpreter 에서 중복되는데, error-prone
                // interpreter 에서 여기 scope 를 가져다 environment 를 생성하게 만들기
                scope
                    .borrow_mut()
                    .variables
                    .insert(stmt.name.clone(), VariableState::Initialized);
                let params_scope = Scope::new_ptr(Some(scope.clone()));
                for p in &stmt.params {
                    params_scope
                        .borrow_mut()
                        .variables
                        .insert(p.into(), VariableState::Initialized);
                }
                self.resolve_statement(&params_scope, &mut stmt.body.write().unwrap())?;
            }
            Statement::Return(stmt) => {
                if let Some(expr) = &mut stmt.value {
                    self.resolve_expression(scope, expr)?;
                }
            }
        }
        Ok(())
    }

    fn resolve_expression(&mut self, scope: &ScopePtr, expr: &mut Expr) -> anyhow::Result<()> {
        match expr {
            Expr::Binary(expr) => {
                self.resolve_expression(scope, &mut expr.left)?;
                self.resolve_expression(scope, &mut expr.right)?;
            }
            Expr::Grouping(expr) => {
                self.resolve_expression(scope, &mut expr.expr)?;
            }
            Expr::Literal(_) => {}
            Expr::Unary(expr) => {
                self.resolve_expression(scope, &mut expr.right)?;
            }
            Expr::Variable(expr) => {
                if let Some(resolution) = scope.borrow().resolve(&expr.name) {
                    expr.resolution = resolution;
                } else {
                    bail!("Referenced undefined varable: {}", expr.name);
                }
            }
            Expr::Assign(expr) => {
                if let Some(resolution) = scope.borrow().resolve(&expr.name) {
                    expr.resolution = resolution;
                } else {
                    bail!("Referenced undefined varable: {}", expr.name);
                }
                self.resolve_expression(scope, &mut expr.value)?;
            }
            Expr::Logical(expr) => {
                self.resolve_expression(scope, &mut expr.left)?;
                self.resolve_expression(scope, &mut expr.right)?;
            }
            Expr::Call(expr) => {
                self.resolve_expression(scope, &mut expr.callee)?;
                for arg in &mut expr.arguments {
                    self.resolve_expression(scope, arg)?;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rlox_parser::{Parser, Scanner};

    fn parse(source: &str) -> anyhow::Result<Vec<Statement>> {
        let tokens = Scanner::new(source).scan_tokens()?;
        let mut stmts = Parser::new(tokens).parse()?;
        Ok(stmts)
    }

    fn resolve(stmts: &mut Vec<Statement>) -> anyhow::Result<()> {
        let mut resolver = Resolver;
        let scope = Scope::new_ptr(None);
        for s in stmts {
            resolver.resolve_statement(&scope, s)?;
        }
        Ok(())
    }

    #[test]
    fn test_static_scope() -> anyhow::Result<()> {
        let source = r#"
var a = "global";
{
    fun showA() {
        print a;
    }
    var a = "block";
    var b = "block";
    print b;
}
        "#;

        let mut stmts = parse(source)?;
        resolve(&mut stmts)?;

        fn visit_statement(stmt: &Statement, print_count: &mut usize) {
            match stmt {
                Statement::Expression(stmt) => {}
                Statement::Print(stmt) => {
                    *print_count += 1;
                    let Expr::Variable(expr ) = &stmt.expr else {
                        panic!("print statement has expr other than variable;")
                    };
                    if expr.name == "a" {
                        assert_eq!(expr.resolution, 3);
                    } else if expr.name == "b" {
                        assert_eq!(expr.resolution, 0);
                    } else {
                        panic!("Weird variable name?")
                    }
                }
                Statement::VariableDecl(stmt) => {}
                Statement::Block(stmt) => {
                    for stmt in &stmt.statements {
                        visit_statement(stmt, print_count);
                    }
                }
                Statement::If(stmt) => {
                    visit_statement(&stmt.then_branch, print_count);
                    if let Some(else_branch) = &stmt.else_branch {
                        visit_statement(else_branch, print_count);
                    }
                }
                Statement::While(stmt) => {
                    visit_statement(&stmt.body, print_count);
                }
                Statement::Function(stmt) => {
                    visit_statement(&stmt.body.read().unwrap(), print_count);
                }
                Statement::Return(stmt) => {}
            }
        }

        let mut print_count = 0;
        for s in &stmts {
            visit_statement(s, &mut print_count);
        }
        assert_eq!(print_count, 2);
        Ok(())
    }

    #[test]
    fn test_variable_resolution_error() {
        // TODO
    }

    #[test]
    fn test_trivial_call_resolution() -> anyhow::Result<()> {
        let source = r#"
fun noop() {}
noop();
        "#;

        let mut stmts = parse(source)?;
        let before = format!("{:?}", stmts);
        resolve(&mut stmts)?;
        let after = format!("{:?}", stmts);
        assert_eq!(before, after);
        Ok(())
    }

    #[test]
    fn test_nested_call_resolution() -> anyhow::Result<()> {
        let source = r#"
fun noop() {}
{ noop(); }
        "#;

        let mut stmts = parse(source)?;
        let before = format!("{:?}", stmts);
        resolve(&mut stmts)?;
        let after = format!("{:?}", stmts);
        assert_ne!(before, after);
        Ok(())
    }

    #[test]
    fn test_no_two_variables_with_same_name_cannot_exist() -> anyhow::Result<()> {
        let source = r#"
var a = 1;
var a = 2;
        "#;

        let mut stmts = parse(source)?;
        let result = resolve(&mut stmts);
        // TODO: enum for error needed to check exactly what the error is.
        assert!(result.is_err());
        assert!(result.err().unwrap().to_string().contains("Already"));
        Ok(())
    }
}
