use core::panic::PanicInfo;

use crate::{ print };

#[panic_handler]
pub fn panic(info: &PanicInfo) -> ! {
    print!("Error: panic: {:#?}", info);
    //print!("panicked");
    loop {}
}