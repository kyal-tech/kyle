use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

static NEXT_TASK_ID: AtomicU64 = AtomicU64::new(1);

pub enum PollState<T> {
    Pending,
    Ready(T),
}

pub type BoxedFuture<T> = Box<dyn FnMut() -> PollState<T> + Send>;

pub struct Task<T> {
    id: u64,
    future: Arc<Mutex<BoxedFuture<T>>>,
}

impl<T> Task<T> {
    pub fn new(future: BoxedFuture<T>) -> Self {
        Self {
            id: NEXT_TASK_ID.fetch_add(1, Ordering::Relaxed),
            future: Arc::new(Mutex::new(future)),
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn poll(&mut self) -> PollState<T> {
        let mut guard = self.future.lock().expect("task lock poisoned");
        (guard)()
    }

    pub fn is_completed(&mut self) -> bool {
        matches!(self.poll(), PollState::Ready(_))
    }
}

unsafe impl<T: Send> Send for Task<T> {}
unsafe impl<T: Send> Sync for Task<T> {}
