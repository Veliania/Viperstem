use core::panic::PanicInfo;

use crate::{ println, print };

#[panic_handler]
pub fn panic(info: &PanicInfo) -> ! {
    println!("Error: panic: {:#?}", info);
    loop {}
}