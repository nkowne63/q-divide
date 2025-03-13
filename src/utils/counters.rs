use std::sync::atomic::{AtomicUsize, Ordering};

// TODO: replace counter by uuid or something which is not global mutable
pub struct Counter {
    counter: AtomicUsize,
}

impl Counter {
    pub const fn new() -> Self {
        Counter {
            counter: AtomicUsize::new(0),
        }
    }

    pub fn increment(&self) -> usize {
        self.counter.fetch_add(1, Ordering::Relaxed)
    }

    pub fn get(&self) -> usize {
        self.counter.load(Ordering::Relaxed)
    }
}