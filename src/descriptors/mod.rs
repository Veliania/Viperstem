#![allow(asm_sub_register)]
use core::mem::size_of;

mod gdt;
use gdt::*;

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

        //calling the new code
        core::arch::asm!(
            "mov rdi, {gdtr}",
            gdtr = in(reg) (&mut DEFAULT_DESC_REG as *mut DescriptorTableRegister) as u64
        );

        load();
    }
}