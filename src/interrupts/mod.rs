use crate::println;

pub mod pic;

pub fn init() {
    pic::init_idt();
    unsafe { pic::PICS.lock().initialize() };
    println!("Interrupts initialized");
}