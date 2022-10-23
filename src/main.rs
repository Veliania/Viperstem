#![no_std]
#![no_main]

use viperstem::*;

// define the kernel's entry point function
#[no_mangle]
extern "C" fn x86_64_barebones_main() -> ! {
    let bootloader_info = BOOTLOADER_INFO
        .get_response()
        .get()
        .expect("barebones: recieved no bootloader info");

    /*println!(
        "bootloader: (name={:?}, version={:?})",
        bootloader_info.name.to_str().unwrap(),
        bootloader_info.version.to_str().unwrap()
    );*/

    //let mmap = MMAP
        //.get_response()
        //.get()
        //.expect("barebones: recieved no mmap")
        //.memmap();

    //println!("mmap: {:#x?}", mmap);

    let lvl5 = LVL5.get_response().get();

    use viperstem::execution::*;

    //match lvl5 {
        //Some(_version) => println!("level 5 paging available, using"),
        //None => println!("level 5 paging not available, using level 4"),
    //}

    kern_push(init); //push the initialization function into the kernel execution loop to demonstrate it
    kern_push(do_smth); //testing to see if a GP fault is thrown

    loop {
        kern_exec(); // begin going through the kernel execution loop
    }
}