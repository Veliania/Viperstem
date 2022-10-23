use crate::{ println, print };

pub static mut RSDP: Option<rsdp::Rsdp> = None;
//pub static mut RSDT: Choice<Option<RSDT>, Option<XSDT>> = crate::cpu::Choice::Empty;

pub static RSDP_REQUEST: limine::LimineRsdpRequest = limine::LimineRsdpRequest::new(0);

pub fn init() {
    unsafe {
        let rsdp_info = RSDP_REQUEST.get_response().get().expect("problem getting RSDP info");

        //println!("{:#?}", rsdp_info);

        let rsdp_ptr = rsdp_info.address.as_ptr().expect("problem getting RSDP from RSDP info");

        //println!("RSDP ptr: {:#?}", rsdp_ptr);

        //println!("{:#?}", *(rsdp_ptr as *const RSDPDescriptor));

        RSDP = Some(*(rsdp_ptr as *const rsdp::Rsdp));

        //print!("Signature: ");
        //for i in 0..8 {
            //print!("{}", RSDP1.unwrap().signature[i] as char);
        //}

        //println!();

        //print!("OEMID: ");
        //for i in 0..6 {
            //print!("{}", RSDP1.unwrap().oemid[i] as char);
        //}
        
        //println!();

        //checksum is ignored by default due to the fact its often not actually used
        /*if (*(rsdp_ptr as *const RSDPDescriptor)).checksum() != true {
            panic!("Checksum invalid: {}", (*(rsdp_ptr as *const RSDPDescriptor)).checksumtotal());
        } else {
            println!("Checksum valid!");
        }*/

        match RSDP.expect("No RSDP provided").revision() {
            0 => println!("ACPI = 1.0"),
            2 => println!("ACPI >= 2.0"),
            _ => panic!("ACPI not supported on device")
        }

        println!("RSDP is valid: {:#?}", RSDP.unwrap().validate() == Ok(()));

        let rsdtptr = RSDP.unwrap().rsdt_address();
    }
}