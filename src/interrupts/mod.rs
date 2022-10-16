use crate::{println, print};

pub fn init() {
    use raw_cpuid::CpuId;
    let cpuid = CpuId::new();

    let apic_status = cpuid.get_feature_info().map_or(false, |finfo| finfo.has_apic());

    match apic_status {
        true => init_apic(),
        _ => panic!("APIC UNSUPPORTED IN CPU, ABORTING"),
    }
}

fn init_apic() {
    panic!("apic code not declared");
}

pub fn call(call_num: u64) {
    use core::arch::asm;
    unsafe { asm!("", in("rax") call_num) };
    unsafe { asm!("int 0x80") };
}

pub fn disable() {
    use core::arch::asm;
    unsafe {asm!("cli");}
}

pub fn enable() {
    use core::arch::asm;
    unsafe {asm!("sti");}
}