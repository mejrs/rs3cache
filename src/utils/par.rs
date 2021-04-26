#[cfg(not(feature = "singlethreaded"))]
use std::{
    sync::{Arc, Mutex},
    thread,
};

/// Enables the [par_apply()](ParApply::par_apply) iterator adapter.
pub trait ParApply: Iterator {
    #[cfg(feature = "singlethreaded")]
    fn par_apply<F>(self, func: F)
    where
        Self: Sized + Send,
        F: Fn(Self::Item) + Sync + Send,
    {
        self.for_each(func);
    }

    /// Calculates `value`s of a `!Sync` iterator and assigns the computation of `func(value)` to the threadpool.
    #[cfg(not(feature = "singlethreaded"))]
    fn par_apply<'f, F>(self, func: F)
    where
        Self: Sized + Send,
        F: Fn(Self::Item) + Sync + Send + 'f,
    {
        let pool_size = num_cpus::get();
        let feed = Arc::new(Mutex::new(self));

        let mut pool = Vec::with_capacity(pool_size);

        for id in 0..pool_size {
            // SAFETY: No reference may outlive the new thread scope...
            let handle = unsafe {
                thread::Builder::new()
                    .name(id.to_string())
                    .spawn_unchecked(|| loop {
                        let item = {
                            match feed.lock() {
                                Ok(mut i) => i.next(),
                                Err(_) => break,
                            }
                        };
                        match item {
                            Some(n) => func(n),
                            None => break,
                        };
                    })
                    .unwrap_or_else(|_| std::process::abort())
            };
            pool.push(handle);
        }

        // SAFETY: ...which is smaller than the enclosing scope as it's joined here...
        for handle in pool {
            handle.join().unwrap_or_else(|_| std::process::abort());
        }

        // SAFETY: ...and the references valid until dropped here.
        drop(func);
    }
}

impl<I: Iterator> ParApply for I {}

#[cfg(test)]
mod tests {
    use super::ParApply;

    #[test]
    fn test_1() {
        fn f(x: usize) {
            println!("{}", x)
        }
        (0..100_usize).par_apply(f);
    }

    #[test]
    #[should_panic]
    fn actually_do_something() {
        fn f(_x: usize) {
            panic!("Hi");
        }
        (0..100_usize).par_apply(f);
    }
}
