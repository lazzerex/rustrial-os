//use core::f128::consts::PI;


use lazy_static::lazy_static;
use pic8259::ChainedPics;
use x86_64::structures::idt::
{InterruptDescriptorTable, InterruptStackFrame};
use spin;
use x86_64::structures::idt::PageFaultErrorCode;
use crate::hlt_loop;
use crate::gdt;
use crate::println;
//use crate::print;



lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX); 
        }
        idt[InterruptIndex::Timer.as_usize()]
            .set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()]
            .set_handler_fn(keyboard_interrupt_handler);
        idt[InterruptIndex::Mouse.as_usize()]
            .set_handler_fn(mouse_interrupt_handler);

        // generate generic irq handlers for PIC IRQs 0..15 and register them
        macro_rules! set_irq_handlers {
            ($idt:ident, $base:expr) => {
                $idt;
                {
                    let mut i = 0u8;
                    while i < 16 {
                        match i {
                            0 => $idt[($base + 0) as usize].set_handler_fn(generic_irq_0),
                            1 => $idt[($base + 1) as usize].set_handler_fn(generic_irq_1),
                            2 => $idt[($base + 2) as usize].set_handler_fn(generic_irq_2),
                            3 => $idt[($base + 3) as usize].set_handler_fn(generic_irq_3),
                            4 => $idt[($base + 4) as usize].set_handler_fn(generic_irq_4),
                            5 => $idt[($base + 5) as usize].set_handler_fn(generic_irq_5),
                            6 => $idt[($base + 6) as usize].set_handler_fn(generic_irq_6),
                            7 => $idt[($base + 7) as usize].set_handler_fn(generic_irq_7),
                            8 => $idt[($base + 8) as usize].set_handler_fn(generic_irq_8),
                            9 => $idt[($base + 9) as usize].set_handler_fn(generic_irq_9),
                            10 => $idt[($base + 10) as usize].set_handler_fn(generic_irq_10),
                            11 => $idt[($base + 11) as usize].set_handler_fn(generic_irq_11),
                            12 => $idt[($base + 12) as usize].set_handler_fn(generic_irq_12),
                            13 => $idt[($base + 13) as usize].set_handler_fn(generic_irq_13),
                            14 => $idt[($base + 14) as usize].set_handler_fn(generic_irq_14),
                            15 => $idt[($base + 15) as usize].set_handler_fn(generic_irq_15),
                            _ => {}
                        }
                        i += 1;
                    }
                }
            };
        }

        set_irq_handlers!(idt, PIC_1_OFFSET);
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

// generated generic irq handlers for 0..15
macro_rules! gen_irq_fn {
    ($name:ident, $num:expr) => {
        extern "x86-interrupt" fn $name(_stack_frame: InterruptStackFrame) {
            // call registered handler for this irq
            handle_registered_irq($num);
            // notify end of interrupt for PIC
            unsafe { PICS.lock().notify_end_of_interrupt(PIC_1_OFFSET + $num); }
        }
    };
}

gen_irq_fn!(generic_irq_0, 0);
gen_irq_fn!(generic_irq_1, 1);
gen_irq_fn!(generic_irq_2, 2);
gen_irq_fn!(generic_irq_3, 3);
gen_irq_fn!(generic_irq_4, 4);
gen_irq_fn!(generic_irq_5, 5);
gen_irq_fn!(generic_irq_6, 6);
gen_irq_fn!(generic_irq_7, 7);
gen_irq_fn!(generic_irq_8, 8);
gen_irq_fn!(generic_irq_9, 9);
gen_irq_fn!(generic_irq_10, 10);
gen_irq_fn!(generic_irq_11, 11);
gen_irq_fn!(generic_irq_12, 12);
gen_irq_fn!(generic_irq_13, 13);
gen_irq_fn!(generic_irq_14, 14);
gen_irq_fn!(generic_irq_15, 15);

extern "x86-interrupt" fn timer_interrupt_handler(
    _stack_frame: InterruptStackFrame) 
{
    //print!(".");
    // call registered irq handlers (irq 0)
    handle_registered_irq(0);
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame, _error_code: u64) -> !
{
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn keyboard_interrupt_handler(
    _stack_frame: InterruptStackFrame)
{
    // use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
    // use spin::Mutex;
    use x86_64::instructions::port::Port;

    // lazy_static! {
    //     static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
    //         Mutex::new(Keyboard::new(ScancodeSet1::new(),
    //             layouts::Us104Key, HandleControl::Ignore)
    //         );
    // }

    // let mut keyboard = KEYBOARD.lock();
    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };
    crate::task::keyboard::add_scancode(scancode);
    // if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
    //     if let Some(key) = keyboard.process_keyevent(key_event) {
    //         match key {
    //             DecodedKey::Unicode(character) => print!("{}", character),
    //             DecodedKey::RawKey(key) => print!("{:?}", key),
    //         }
    //     }
    // }

    // call registered irq handlers (irq 1)
    handle_registered_irq(1);
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}

extern "x86-interrupt" fn mouse_interrupt_handler(
    _stack_frame: InterruptStackFrame)
{
    use x86_64::instructions::port::Port;
    
    let mut port = Port::new(0x60);
    let packet_byte: u8 = unsafe { port.read() };
    
    crate::task::mouse::add_byte(packet_byte);
    
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Mouse.as_u8());
    }
}

// irq handler registry for dynamic registration (supports 16 pic irqs)
use core::option::Option;
use spin::Mutex as SpinMutex;

static IRQ_HANDLERS: SpinMutex<[Option<fn()>; 16]> = SpinMutex::new([None; 16]);

// register a simple handler for a given irq number (0-15)
pub fn register_irq_handler(irq: u8, handler: fn()) {
    let idx = irq as usize;
    if idx < 16 {
        IRQ_HANDLERS.lock()[idx] = Some(handler);
    }
}

// internal helper called from irq entry points
fn handle_registered_irq(irq: u8) {
    let idx = irq as usize;
    if idx < 16 {
        if let Some(h) = IRQ_HANDLERS.lock()[idx] {
            h();
        }
    }
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;

    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error Code: {:?}", error_code);
    println!("{:#?}", stack_frame);
    hlt_loop();
}

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: spin::Mutex<ChainedPics> = spin::Mutex::new(
    unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) }
);

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
    Mouse = PIC_2_OFFSET + 4, // IRQ 12 (secondary PIC)
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}


#[test_case]
fn test_breakpoint_exception() {
    x86_64::instructions::interrupts::int3();
}