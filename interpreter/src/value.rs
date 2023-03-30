use crate::func::{FunctionObject, NativeFunction};
use std::fmt::Debug;
use std::ops::Deref;
use std::sync::Arc;
use syntax_tree::Literal;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    String(String),
    Boolean(bool),
    Nil,
    NativeFunction(&'static NativeFunction),
    // TODO: object - garbage collection, etc.
    FunctionObject(Object<FunctionObject>),
}

impl From<Literal> for Value {
    fn from(value: Literal) -> Self {
        match value {
            Literal::Number(value) => Self::Number(value),
            Literal::String(value) => Self::String(value),
            Literal::Boolean(value) => Self::Boolean(value),
            Literal::Nil => Self::Nil,
        }
    }
}

// Arc is necessary due to the current implementation of return statement using anyhow::Error.
#[derive(Debug)]
pub struct Object<T: Debug>(Arc<T>);

impl<T: Debug> Object<T> {
    pub fn new(payload: T) -> Self {
        Self(Arc::new(payload))
    }
}

impl<T: Debug> Clone for Object<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: Debug> Deref for Object<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Debug> PartialEq for Object<T> {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl<T: Debug> AsRef<T> for Object<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}
