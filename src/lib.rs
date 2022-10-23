#![feature(abi_x86_interrupt, core_intrinsics)]
#![no_std]

pub mod output;
pub mod cpu;
pub mod panic;
pub mod descriptors;
pub mod execution;

pub use limine::*;

pub static TERMINAL_REQUEST: LimineTerminalRequest = LimineTerminalRequest::new(0);
pub static BOOTLOADER_INFO: LimineBootInfoRequest = LimineBootInfoRequest::new(0);
pub static MMAP: LimineMemmapRequest = LimineMemmapRequest::new(0);
pub static LVL5: Limine5LevelPagingRequest = Limine5LevelPagingRequest::new(0);

pub fn init() {
    println!("Initializing CPU aspects");
    cpu::init();
    println!("CPU aspects initialized");
    println!("Initializing GDT aspects");
    descriptors::init();
    //println!("GDT aspects initialized"); //running this causes errors for the system
}

static mut thingy: Option<i32> = None;

pub fn do_smth() {
    let a = 10+10;
    let b = a + a;
    let c = b + b;

    unsafe {thingy = Some(c);}
}

#[macro_export]
macro_rules! panic {
    ($($arg:tt)*) => ($crate::panic::panic(format_args!($($arg)*), ));
}

pub fn hlt_loop() -> ! {
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}