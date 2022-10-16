#![no_std]
#![no_main]

use viperstem::*;

// define the kernel's entry point function
#[no_mangle]
extern "C" fn x86_64_barebones_main() -> ! {
    println!("Hello, rusty world!");

    let _bootloader_info = BOOTLOADER_INFO
        .get_response()
        .get()
        .expect("barebones: recieved no bootloader info");

    /*println!(
        "bootloader: (name={:?}, version={:?})",
        bootloader_info.name.to_str().unwrap(),
        bootloader_info.version.to_str().unwrap()
    );*/

    let _mmap = MMAP
        .get_response()
        .get()
        .expect("barebones: recieved no mmap")
        .mmap();

    //println!("mmap: {:#x?}", mmap);

    init();

    loop {}
}
