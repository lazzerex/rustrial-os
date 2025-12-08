use core::{future::Future, pin::Pin};
use alloc::boxed::Box;
use core::task::{Context, Poll};
use core::sync::atomic::{AtomicU64, Ordering};

pub mod simple_executor;
pub mod keyboard;
pub mod executor;
pub mod mouse;
    pub mod preemptive;

    /// Context switch stub, to be implemented with FFI to assembly
    extern "C" {
        fn switch_context_asm();
    }

    pub fn context_switch() {
        // Select next ready task (stub)
        if let Some(next_idx) = crate::task::preemptive::select_next_task() {
            // TODO: Save current PCB, load next PCB, and call assembly routine
            unsafe {
                switch_context_asm();
            }
        }
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
