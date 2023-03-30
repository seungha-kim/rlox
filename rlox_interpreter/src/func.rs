use crate::interpreter::{Environment, Interpreter};
use crate::value::Value;
use anyhow::bail;
use rlox_syntax::Statement;
use std::cmp::Ordering;
use std::fmt::{Debug, Formatter};
use std::sync::{Arc, Mutex};

pub trait Callable {
    fn arity(&self) -> usize;
    fn call(&self, interpreter: &mut Interpreter, args: &[Value]) -> anyhow::Result<Value>;
}

pub struct FunctionObject {
    pub name: String,
    pub parameters: Vec<String>,
    pub body: Arc<Statement>,
    pub closure: Arc<Mutex<Environment>>,
}

impl Debug for FunctionObject {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "FunctionObject({:?})", self.name)
    }
}

impl Callable for FunctionObject {
    fn arity(&self) -> usize {
        self.parameters.len()
    }

    fn call(&self, interpreter: &mut Interpreter, args: &[Value]) -> anyhow::Result<Value> {
        match args.len().cmp(&self.arity()) {
            Ordering::Less => bail!("More args must be given"),
            Ordering::Greater => bail!("Less args must be given"),
            _ => {}
        }

        let environment = Environment::new_ptr(self.closure.clone());
        {
            let mut env = environment.lock().unwrap();
            for (param, arg) in self.parameters.iter().zip(args.iter()) {
                // TODO: do not clone
                env.define_variable(param, arg.clone())?;
            }
        }
        interpreter.evaluate_stmt(&environment, &self.body)?;

        Ok(Value::Nil)
    }
}

type NativeFuncPtr = fn(&mut Interpreter, &[Value]) -> anyhow::Result<Value>;

pub struct NativeFunction {
    pub name: &'static str,
    pub arity: usize,
    pub func: NativeFuncPtr,
}

impl Debug for NativeFunction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "NativeFunction({})", self.func as usize)
    }
}

impl PartialEq for NativeFunction {
    fn eq(&self, other: &Self) -> bool {
        self.arity == other.arity && self.func as usize == other.func as usize
    }
}

impl Callable for NativeFunction {
    fn arity(&self) -> usize {
        self.arity
    }

    fn call(&self, interpreter: &mut Interpreter, args: &[Value]) -> anyhow::Result<Value> {
        (self.func)(interpreter, args)
    }
}

pub mod impls {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    pub static CLOCK: NativeFunction = NativeFunction {
        name: "clock",
        arity: 2,
        func: |_interpreter, _args| {
            Ok(Value::Number(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs_f64(),
            ))
        },
    };

    pub static ALL_FUNCS: &[&NativeFunction] = &[&CLOCK];
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpreter::StdOutPrinter;

    static HELLO: NativeFunction = NativeFunction {
        name: "hello",
        arity: 0,
        func: |_interpreter: &mut Interpreter, _args: &[Value]| Ok(Value::Nil),
    };

    #[test]
    fn test_call() {
        let mut printer = StdOutPrinter;
        let mut interpreter = Interpreter::new(&mut printer);
        HELLO.call(&mut interpreter, &[]).unwrap();
    }

    #[test]
    fn test_equal() {
        let f1 = Value::NativeFunction(&impls::CLOCK);
        let f2 = Value::NativeFunction(&impls::CLOCK);
        assert_eq!(f1, f2);
    }
}
