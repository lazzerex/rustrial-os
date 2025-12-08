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
        fn switch_context_asm(current: *mut crate::task::preemptive::PCB, next: *const crate::task::preemptive::PCB);
    }

    pub fn context_switch() {
        use crate::task::preemptive::{CURRENT_TASK, TASKS, select_next_task};
        let mut tasks = TASKS.lock();
        let current_idx = CURRENT_TASK.load(core::sync::atomic::Ordering::Relaxed);
        if let Some(next_idx) = select_next_task() {
            if next_idx != current_idx {
                let current_pcb = &mut tasks[current_idx] as *mut _;
                let next_pcb = &tasks[next_idx] as *const _;
                CURRENT_TASK.store(next_idx, core::sync::atomic::Ordering::Relaxed);
                unsafe {
                    switch_context_asm(current_pcb, next_pcb);
                }
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
