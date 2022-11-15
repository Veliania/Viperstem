#![no_std]
#![no_main]

use core::alloc::Layout;
extern crate alloc;

use viperstem::{ *, bitmap::BitMap, memory::sect_alloc::{ sect_count, space, sect_size, is_used, req_sect_size, ret_sect }, output::FRAME_REQUEST };

// define the kernel's entry point function
#[no_mangle]
extern "C" fn kernel_main() -> ! {
    init();

    let bootloader_info = BOOTLOADER_INFO
        .get_response()
        .get()
        .expect("barebones: recieved no bootloader info");

    println!(
        "bootloader: (name={:?}, version={:?})",
        bootloader_info.name.to_str().unwrap(),
        bootloader_info.version.to_str().unwrap()
    );

    /*let mmap = MMAP
        .get_response()
        .get()
        .expect("barebones: recieved no mmap");*/

    //println!("mmap: {:#x?}", mmap);

    let lvl5 = LVL5.get_response().get();


    match lvl5 {
        Some(_version) => println!("level 5 paging available, using"),
        None => println!("level 5 paging not available, using level 4"),
    }

    let mut executor = task::executor::Executor::new();

    executor.spawn(task::Task::new(_tests()));

    executor.run();
}

async fn _tests() {
    unsafe {
        let mythingyptr = alloc::alloc::alloc(Layout::new::<u64>()) as *mut u64;

        *mythingyptr = 112;

        assert!(*mythingyptr == 112, "Invalid value at pointer, should be 112");

        println!("time at start: {}", UnixTime::from_i64(TIME.get_response().get().expect("Could not get time").boot_time));

        let mut buf: [u8; 4] = [0; 4];

        let bitmap = BitMap::new(4, &mut buf[0]);

        let mut state = false;

        for i in 0..32 {
            bitmap.set_bool(i, state);
            state = !state;
        }
        
        state = false;

        for i in 0..32 {
            assert!(*bitmap.get_bool(i) == state, "Bitmap boolean is incorrect value, should have been: {}", state);
            state = !state;
        }

        let mut myvec = alloc::vec::Vec::new();

        myvec.push(42);

        assert!(myvec[0] == 42, "Vector number was not stored properly");
        println!("Tests succeeded");
        
        let (level_4_page_table, _) = x86_64::registers::control::Cr3::read();
        println!("Level 4 page table at: {:?}", level_4_page_table.start_address());

        println!("{:#?} MiB", (space() as f64) / 1_048_576.0);
        println!("{:#?} useable sections", sect_count());
        
        let index = req_sect_size(4096).1;
        ret_sect(index);

        println!();

        for i in 0..sect_count() as usize {
            println!("sect[{}] size = {}", i, sect_size(i));
            println!("Section takes up {} pages", sect_size(i) as f64 / 4096 as f64);
            println!("Section is{}used\n", match is_used(i) {
                true => " ",
                false => " not "
            });
        }

        println!("size of frame buffer: {}", FRAME_REQUEST.get_response().get().expect("failed to get frame buffer").framebuffers()[0].size());

        for entry in MMAP.get_response().get().unwrap().memmap() {
            match entry.typ {
                LimineMemoryMapEntryType::Framebuffer => println!("Found framebuffer"),
                _ => ()
            }
        }

        let mmap = MMAP.get_response().get().unwrap();
        let mut size = 0;

        for entry in mmap.memmap() {
            if (entry.base + entry.len) > size {
                size = entry.base + entry.len;
            }
        }

        println!("mem size is {}", size);

        paging::paging();
    }
}