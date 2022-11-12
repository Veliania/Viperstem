//use aligned::{Aligned, A8};

#[repr(C, packed)]
pub struct DescriptorTableRegister {
    pub limit: u16,
    pub offset: u64
}

#[repr(C, packed)]
pub struct GDT {
    pub null: GDTEntry,
    /*code16: GDTEntry,
    data16: GDTEntry,
    code32: GDTEntry,
    data32: GDTEntry,
    code64: GDTEntry,
    data64: GDTEntry,*/
    pub kernel_code: GDTEntry,
    pub kernel_data: GDTEntry,
    pub user_code: GDTEntry,
    pub user_data: GDTEntry,
}

impl GDT {
    //supports all segments
    /*const fn null() -> GDT {
        GDT { null: GDTEntry::null(), code16: GDTEntry::null(), data16: GDTEntry::null(), code32: GDTEntry::null(), data32: GDTEntry::null(), code64: GDTEntry::null(), data64: GDTEntry::null(), kernel_code: GDTEntry::null(), kernel_data: GDTEntry::null(), user_code: GDTEntry::null(), user_data: GDTEntry::null(), }
    }*/
    
    //only supports kernel and user segments
    pub const fn null() -> GDT {
        GDT { null: GDTEntry::null(), kernel_code: GDTEntry::null(), kernel_data: GDTEntry::null(), user_code: GDTEntry::null(), user_data: GDTEntry::null(), }
    }
    
    //does not support kernel and user segments
    /*const fn null() -> GDT {
        GDT { null: GDTEntry::null(), code16: GDTEntry::null(), data16: GDTEntry::null(), code32: GDTEntry::null(), data32: GDTEntry::null(), code64: GDTEntry::null(), data64: GDTEntry::null() }
    }*/
}

#[repr(C, packed)]
pub struct GDTEntry {
    limit0: u16,
    base0: u16,
    base1: u8,
    access_byte: u8,
    limit1_and_flags: u8,
    base2: u8
}

impl GDTEntry {
    pub const fn null() -> GDTEntry {
        GDTEntry {
            limit0: 0, 
            base0: 0, 
            base1: 0, 
            access_byte: 0, 
            limit1_and_flags: 0, 
            base2: 0
        }
    }

    pub const fn new(access_byte: u8, flags: u8) -> GDTEntry {
        GDTEntry {
            limit0: 0, 
            base0: 0, 
            base1: 0, 
            access_byte: access_byte, 
            limit1_and_flags: flags << 4, 
            base2: 0
        }
    }

    fn _base_new(access_byte: u8, flags: u8, limit: u32, base: u32) -> GDTEntry {
        let mut new = GDTEntry {
            limit0: 0, 
            base0: 0, 
            base1: 0, 
            access_byte: access_byte, 
            limit1_and_flags: flags << 4, 
            base2: 0
        };

        new.set_limit(limit);
        new.set_base(base);

        return new;
    }

    #[allow(dead_code)]
    fn limit_new(access_byte: u8, flags: u8, limit: u32) -> GDTEntry {
        let mut new = GDTEntry {
            limit0: 0, 
            base0: 0, 
            base1: 0, 
            access_byte: access_byte, 
            limit1_and_flags: flags << 4, 
            base2: 0
        };

        new.set_limit(limit);

        return new;
    }

    #[allow(dead_code)]
    fn set_limit(&mut self, limit: u32) {
        self.limit0 = (limit & 0xFFFF) as u16;
        self.limit1_and_flags |= (limit >> 16) as u8 & 0xF;
    }

    #[allow(dead_code)]
    fn set_base(&mut self, base: u32) {
        self.base0 = (base & 0xFFFF) as u16;
        self.base1 = ((base >> 16) & 0xFF) as u8;
        self.base2 = ((base >> 24) & 0xFF) as u8;
    }
}

//static mut DEFAULT_DESCRIPTOR: Aligned<A8, GDT> = Aligned(GDT::null()); //aligned
pub static mut DEFAULT_DESCRIPTOR: GDT = GDT::null(); //unaligned

pub static mut DEFAULT_DESC_REG: DescriptorTableRegister = DescriptorTableRegister {
    limit: core::mem::size_of::<GDT>() as u16,
    offset: 0,
};