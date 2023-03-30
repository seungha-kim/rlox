use std::sync::atomic::AtomicUsize;

pub use rlox_derive::*;

pub type Ptr<T> = Box<T>;

static ID: AtomicUsize = AtomicUsize::new(0);

pub trait SyntaxNode {
    fn id(&self) -> usize;

    fn generate_id() -> usize {
        let result = ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        if result == usize::MAX {
            panic!("ID overflow");
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    #[test]
    fn test_syntax_node() {
        assert_eq!(expr::Unary::generate_id(), 0);
        assert_eq!(expr::Binary::generate_id(), 1);
        assert_eq!(statement::If::generate_id(), 2);
    }
}
