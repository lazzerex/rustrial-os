
use alloc::vec::Vec;
use core::sync::atomic::{AtomicUsize, Ordering};
use spin::Mutex;

static NEXT_TASK: AtomicUsize = AtomicUsize::new(0);
static CURRENT_TASK: AtomicUsize = AtomicUsize::new(0);
static TASKS: Mutex<Vec<PCB>> = Mutex::new(Vec::new());
static TICK_COUNT: AtomicUsize = AtomicUsize::new(0);

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
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub r11: u64,
    pub r10: u64,
    pub r9: u64,
    pub r8: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rbp: u64,
    pub rdx: u64,
    pub rcx: u64,
    pub rbx: u64,
    pub rax: u64,
    pub rsp: u64,
    pub rip: u64,
    pub rflags: u64,
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


// Called by timer interrupt to increment tick count (atomic)
pub fn tick() {
    TICK_COUNT.fetch_add(1, Ordering::Relaxed);
}

// Called by timer interrupt to check if time slice expired and trigger context switch
pub fn maybe_switch_task() -> bool {
    if TICK_COUNT.load(Ordering::Relaxed) >= TIME_SLICE_TICKS {
        TICK_COUNT.store(0, Ordering::Relaxed);
        super::context_switch();
        return true;
    }
    false
}