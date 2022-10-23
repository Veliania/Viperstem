use core::mem::size_of;

use aligned::{Aligned, A8};

#[repr(C, packed)]
struct DescriptorTableRegister {
    limit: u16,
    offset: u64
}

#[repr(C, packed)]
struct GDT {
    null: GDTEntry,
    kernel_code: GDTEntry,
    kernel_data: GDTEntry,
    user_code: GDTEntry,
    user_data: GDTEntry,
}

impl GDT {
    const fn null() -> GDT {
        GDT { null: GDTEntry::null(), kernel_code: GDTEntry::null(), kernel_data: GDTEntry::null(), user_code: GDTEntry::null(), user_data: GDTEntry::null() }
    }
}

#[repr(C, packed)]
struct GDTEntry {
    limit0: u16,
    base0: u16,
    base1: u8,
    access_byte: u8,
    limit1_and_flags: u8,
    base2: u8
}

impl GDTEntry {
    const fn null() -> GDTEntry {
        GDTEntry {
            limit0: 0, 
            base0: 0, 
            base1: 0, 
            access_byte: 0, 
            limit1_and_flags: 0, 
            base2: 0
        }
    }

    const fn new(access_byte: u8, flags: u8) -> GDTEntry {
        GDTEntry {
            limit0: 0, 
            base0: 0, 
            base1: 0, 
            access_byte: access_byte, 
            limit1_and_flags: flags << 4, 
            base2: 0
        }
    }
}

//static mut DEFAULT_DESCRIPTOR: Aligned<A8, GDT> = Aligned(GDT::null()); //aligned
static mut DEFAULT_DESCRIPTOR: GDT = GDT::null(); //unaligned

static mut DEFAULT_DESC_REG: DescriptorTableRegister = DescriptorTableRegister {
    limit: core::mem::size_of::<GDT>() as u16,
    offset: 0,
};

extern "C" {
    fn load();
}

pub fn init() {
    unsafe {
        DEFAULT_DESCRIPTOR = GDT { 
            null: GDTEntry::null(), 
            kernel_code: GDTEntry::new(0b1_00_1_1_0_1_0, 0b1_0_1_0), 
            kernel_data: GDTEntry::new(0b1_00_1_0_0_1_0, 0b1_0_1_0), 
            user_code: GDTEntry::new(0b1_11_1_1_0_1_0, 0b1_0_1_0), 
            user_data: GDTEntry::new(0b1_11_1_0_0_1_0, 0b1_0_1_0) 
        };

        DEFAULT_DESC_REG = DescriptorTableRegister {
            limit: size_of::<GDT>() as u16,
            offset: (&mut DEFAULT_DESCRIPTOR as *mut GDT) as u64
        };

        //original code to load GDT
        /*core::arch::asm!(
            "lgdt [{gdtr}]",
            //"retfq",
            gdtr = in(reg) (&mut DEFAULT_DESC_REG as *mut DescriptorTableRegister) as u64
        );*/

        //calling the new code
        core::arch::asm!(
            "mov rdi, {gdtr}",
            //"retfq",
            gdtr = in(reg) (&mut DEFAULT_DESC_REG as *mut DescriptorTableRegister) as u64
        );

        load();
    }
}