use core::ops::Deref;
use std::sync::OnceLock;

#[derive(Debug)]
pub struct Lazy<A: Clone, T, E> {
    args: A,
    f: fn(A) -> Result<T, E>,
    ready: OnceLock<Result<T, E>>,
}

impl<A: Clone, T, E> Lazy<A, T, E> {
    pub fn new(args: A, f: fn(A) -> Result<T, E>) -> Self {
        Self {
            args,
            f,
            ready: OnceLock::new(),
        }
    }

    pub fn get(&self) -> &Result<T, E> {
        self.ready.get_or_init(|| (self.f)(self.args.clone()))
    }
}

impl<A: Clone, T, E> Deref for Lazy<A, T, E> {
    type Target = Result<T, E>;
    fn deref(&self) -> &Self::Target {
        self.get()
    }
}
