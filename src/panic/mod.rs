use core::panic::PanicInfo;

use crate::{ output, print };

#[doc(hidden)]
#[panic_handler]
pub fn _panic(info: &PanicInfo) -> ! {
    output::set_colors(0xFF0000, 0xFFFFFFFF);
    print!("Error: panic: {:#?}", info);
    loop {}
}

use core::alloc::Layout;

#[doc(hidden)]
#[alloc_error_handler]
pub fn _alloc_panic(info: Layout) -> ! {
    output::set_colors(0xFF0000, 0xFFFFFFFF);
    print!("alloc error: {:#?}", info);
    loop {}
}