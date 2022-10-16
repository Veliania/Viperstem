#![no_std]

pub mod output;
pub mod interrupts;
pub mod cpu;
pub mod panic;

pub use limine::*;

pub static TERMINAL_REQUEST: LimineTerminalRequest = LimineTerminalRequest::new(0);
pub static BOOTLOADER_INFO: LimineBootInfoRequest = LimineBootInfoRequest::new(0);
pub static MMAP: LimineMemmapRequest = LimineMemmapRequest::new(0);

pub fn init() {
    cpu::init();
    interrupts::init();
}


#[macro_export]
macro_rules! panic {
    ($($arg:tt)*) => ($crate::panic::panic(format_args!($($arg)*), ));
}