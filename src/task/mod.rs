use core::{future::Future, pin::Pin};
use alloc::boxed::Box;
use core::task::{Context, Poll};
use core::sync::atomic::{AtomicU64, Ordering};
use spin::Mutex;
use alloc::vec::Vec;

pub mod simple_executor;
pub mod keyboard;
pub mod executor;
pub mod mouse;

/// Global task storage for spawning background tasks
static GLOBAL_TASKS: Mutex<Vec<Task>> = Mutex::new(Vec::new());

/// Spawn a task to be picked up by the executor
pub fn spawn_task(future: impl Future<Output = ()> + 'static) {
    let task = Task::new(future);
    GLOBAL_TASKS.lock().push(task);
}

/// Get all pending tasks (called by executor)
pub fn take_pending_tasks() -> Vec<Task> {
    let mut tasks = GLOBAL_TASKS.lock();
    let pending = tasks.drain(..).collect();
    pending
}

pub struct Task {
    id: TaskId,
    future: Pin<Box<dyn Future<Output = ()>>>,
}

impl Task {
    pub fn new(future: impl Future<Output = ()> + 'static) -> Task {
        Task {
            id: TaskId::new(),
            future: Box::pin(future),
        }
    }

    fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}

impl TaskId {
    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        TaskId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct TaskId(u64);

/// Yields execution to allow other tasks to run
///
/// This creates a future that returns Poll::Pending once, then Poll::Ready
/// on the next poll, effectively yielding to the scheduler.
pub async fn yield_now() {
    struct YieldNow {
        yielded: bool,
    }

    impl Future for YieldNow {
        type Output = ();

        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
            if self.yielded {
                Poll::Ready(())
            } else {
                self.yielded = true;
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        }
    }

    YieldNow { yielded: false }.await
}
