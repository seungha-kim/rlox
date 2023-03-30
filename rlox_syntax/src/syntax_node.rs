pub use uuid::Uuid;

pub use rlox_derive::*;

pub type Ptr<T> = Box<T>;

pub trait SyntaxNode {
    fn id(&self) -> Uuid;
}
