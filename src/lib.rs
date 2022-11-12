#![allow(non_snake_case)]
#![recursion_limit = "256"]
#![feature(custom_test_frameworks)]
#![feature(alloc_error_handler)]
//#![feature(alloc_layout_extra)]
//#![feature(type_alias_impl_trait)]
#![feature(abi_x86_interrupt, core_intrinsics)]
#![no_std]

extern crate alloc;

//pub mod arch;
pub mod memory;
pub mod paging;
pub mod structures;
pub mod output;
pub mod panic;
pub mod descriptors;
pub mod interrupts;
pub mod task;
pub mod allocator;
pub mod utils;

pub use limine::*;
use memory::sect_alloc::{req_sect_size, empty_section};
pub use structures::*;
use x86_64::{structures::paging::Size4KiB, VirtAddr };

//pub static TERMINAL_REQUEST: LimineTerminalRequest = LimineTerminalRequest::new(0);
pub static BOOTLOADER_INFO: LimineBootInfoRequest = LimineBootInfoRequest::new(0);
pub static MMAP: LimineMemmapRequest = LimineMemmapRequest::new(0);
pub static LVL5: Limine5LevelPagingRequest = Limine5LevelPagingRequest::new(0);
pub static TIME: LimineBootTimeRequest = LimineBootTimeRequest::new(0);
pub static RSDP: LimineRsdpRequest = LimineRsdpRequest::new(0);

pub fn init() {
    output::init();
    descriptors::init();
    interrupts::init();
    x86_64::instructions::interrupts::enable();
    memory::sect_alloc::init_sect_manager();
    unsafe {
        let section = req_sect_size(4096);
        let mut page = x86_64::structures::paging::Page::<Size4KiB>::from_start_address_unchecked(VirtAddr::from_ptr(section.0.base));
        empty_section(section.1);
        allocator::init_heap(&mut page);
    }
}

pub fn hlt_loop() -> ! {
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}

pub struct UnixTime(u32);

impl UnixTime {
    pub fn from_i64(time: i64) -> UnixTime {
        UnixTime(time as u32)
    }

    pub fn secs(&self) -> u32 {
        self.0 % 60
    }

    pub fn pure_secs(&self) -> u32 {
        self.0 % 60
    }

    pub fn mins(&self) -> u32 {
        ((self.0 - self.pure_secs()) % 3600) / 60
    }

    pub fn pure_mins(&self) -> u32 {
        (self.0 - self.pure_secs()) % 3600
    }

    pub fn hrs(&self) -> u32 {
        (((self.0 - self.pure_secs()) - self.pure_mins()) % 86_400) / 3600
    }

    pub fn pure_hrs(&self) -> u32 {
        ((self.0 - self.pure_secs()) - self.pure_mins()) % 86_400
    }

    pub fn days(&self) -> u32 {
        ((((self.0 - self.pure_secs()) - self.pure_mins()) - self.pure_hrs()) % 2_682_000) / 86_400
    }

    pub fn pure_days(&self) -> u32 {
        (((self.0 - self.pure_secs()) - self.pure_mins()) - self.pure_hrs()) % 2_682_000
    }

    pub fn mths(&self) -> u32 {
        (((((self.0 - self.pure_secs()) - self.pure_mins()) - self.pure_hrs()) - self.pure_days()) % 31_536_000) / 2_682_000
    }

    pub fn pure_mths(&self) -> u32 {
        ((((self.0 - self.pure_secs()) - self.pure_mins()) - self.pure_hrs()) - self.pure_days()) % 31_536_000
    }

    pub fn yrs(&self) -> u32 {
        (((((self.0 - self.pure_secs()) - self.pure_mins()) - self.pure_hrs()) - self.pure_days()) - self.pure_mths()) / 31_536_000
    }

    pub fn pure_yrs(&self) -> u32 {
        ((((self.0 - self.pure_secs()) - self.pure_mins()) - self.pure_hrs()) - self.pure_days()) - self.pure_mths()
    }
}

use core::fmt;

impl fmt::Display for UnixTime {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Write strictly the first element into the supplied output
        // stream: `f`. Returns `fmt::Result` which indicates whether the
        // operation succeeded or failed. Note that `write!` uses syntax which
        // is very similar to `println!`.
        let seconds = self.secs();
        let minutes = self.mins();
        let hours = self.hrs() / 12;
        let days = self.days();
        let months = self.mths();
        let years = self.yrs() + 1970;
        let am_pm = match self.hrs() > 12 {
            true => "PM",
            false => "AM"
        };

        write!(f, "{}/{}/{} {}:{}:{} {}", days, months, years, hours, minutes, seconds, am_pm)
    }
}