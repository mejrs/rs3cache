use std::{
    sync::{Arc, Mutex},
    thread,
};

/// Enables the [par_apply()](ParApply::par_apply) iterator adapter.
pub trait ParApply: Iterator {
    /// Calculates `value`s of a `!Sync` iterator and assigns the computation of `func(value)` to the threadpool.
    fn par_apply<'f, F>(self, func: F)
    where
        Self: Sized + Send,
        F: Fn(Self::Item) + Sync + Send + 'f,
    {
        let pool_size = num_cpus::get();
        let feed = Arc::new(Mutex::new(self));

        let mut pool = Vec::with_capacity(pool_size);

        for _ in 0..pool_size {
            // SAFETY: No reference may outlive the new thread scope...
            let handle = unsafe {
                thread::Builder::new().spawn_unchecked(|| loop {
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
            };
            // avoid panicking
            if let Ok(h) = handle {
                pool.push(h);
            } else {
                println!("Error creating thread");
            }
        }

        // SAFETY: ...which is smaller than the enclosing scope as it's joined here...
        let joined = pool.into_iter().map(|h| h.join()).collect::<Vec<_>>();

        for join in joined {
            // now it's fine to panic if necessary
            join.unwrap();
        }

        drop(feed);
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
