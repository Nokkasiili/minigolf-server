use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

pub struct Inner {
    next_id: AtomicUsize,
}
#[derive(Clone)]

pub struct IdGenerator {
    inner: Arc<Inner>,
}

impl IdGenerator {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Inner {
                next_id: AtomicUsize::new(1),
            }),
        }
    }

    pub fn next_id(&mut self) -> usize {
        self.inner.next_id.fetch_add(1, Ordering::SeqCst)
    }
}
