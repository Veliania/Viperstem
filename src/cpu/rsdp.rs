#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct RSDPDescriptor {
    signature: [char; 8],
    checksum: u8,
    oemid: [char; 6],
    pub revision: u8,
    rsdt_address: u32,
}

#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct RSDPDescriptor2 {
    first: RSDPDescriptor,
    length: u32,
    xsdt_address: u64,
    extended_checksum: u8,
    reserved: [u8; 3],
}