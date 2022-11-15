use x86_64::instructions::port::Port;

use crate::println;

pub mod pic;

pub fn init() {
    pic::init_idt();
    unsafe { pic::PICS.lock().initialize() };
    println!("Interrupts initialized");

    /*for i in 0..255 {
        clear_mask(i);
    }*/
    clear_mask(1);
}

fn clear_mask(irqline: u8) {
    unsafe {
        let mut port = match irqline >= 8{
            true => Port::new(0xA1),
            false => Port::new(0x21)
        };
        
        let value: u8 = port.read() & !((1 as u8).checked_shl(irqline as u32).unwrap_or(0));
        port.write(value);
    }
}

fn set_mask(irqline: u8) {
    unsafe {
        let mut port = match irqline >= 8{
            true => Port::new(0xA1),
            false => Port::new(0x21)
        };
        
        let value: u8 = port.read() | ((1 as u8).checked_shl(irqline as u32).unwrap_or(0));
        port.write(value);
    }
}