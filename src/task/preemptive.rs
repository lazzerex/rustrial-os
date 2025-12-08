use alloc::vec::Vec;
use core::sync::atomic::{AtomicUsize, Ordering};
use spin::Mutex;

static NEXT_TASK: AtomicUsize = AtomicUsize::new(0);
static TASKS: Mutex<Vec<PCB>> = Mutex::new(Vec::new());

/// Add a new task to the scheduler
pub fn add_task(pcb: PCB) {
    TASKS.lock().push(pcb);
}

/// Select the next ready task (round-robin)
pub fn select_next_task() -> Option<usize> {
    let tasks = TASKS.lock();
    let n = tasks.len();
    if n == 0 { return None; }
    let mut idx = NEXT_TASK.load(Ordering::Relaxed);
    for _ in 0..n {
        idx = (idx + 1) % n;
        if tasks[idx].state == TaskState::Ready {
            NEXT_TASK.store(idx, Ordering::Relaxed);
            return Some(idx);
        }
    }
    None
}
// Preemptive multitasking core logic
use core::arch::asm;

pub const TIME_SLICE_TICKS: usize = 10;
static mut TICK_COUNT: usize = 0;

#[repr(C)]
pub struct PCB {
    pub regs: [u64; 16], // General purpose registers, e.g., rax, rbx, rcx, rdx, rsi, rdi, rbp, rsp, r8-r15
    pub rip: u64,        // Instruction pointer
    pub rsp: u64,        // Stack pointer
    pub state: TaskState,
    // Add more fields as needed (e.g., FPU state, PID, etc.)
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Ready,
    Running,
    Blocked,
    Terminated,
}

// Called by timer interrupt to increment tick count
pub fn tick() {
    unsafe {
        TICK_COUNT += 1;
    }
}

// Called by timer interrupt to check if time slice expired and trigger context switch
pub fn maybe_switch_task() -> bool {
    unsafe {
        if TICK_COUNT >= TIME_SLICE_TICKS {
            TICK_COUNT = 0;
            // Call context switch (to be implemented)
            super::context_switch();
            return true;
        }
    }
    false
}