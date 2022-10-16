pub mod rsdp;

use crate::{ println, print };
use rsdp::*;

static mut RSDP: Option<rsdp::RSDPDescriptor> = None;
static mut RSDP2: Option<rsdp::RSDPDescriptor2> = None;

pub static RSDP_REQUEST: limine::LimineRsdpRequest = limine::LimineRsdpRequest::new(0);

pub fn init() {
    unsafe {
        let rsdp_info = RSDP_REQUEST.get_response().get().expect("problem getting RSDP info");

        println!("{:#?}", rsdp_info);

        let rsdp_ptr = rsdp_info.address.as_ptr().expect("problem getting RSDP from RSDP info");

        println!("{:#?}", *(rsdp_ptr as *const RSDPDescriptor));

        /*match buffer.unwrap().revision {
            0 => RSDP = buffer,
            2 => RSDP2 = Some(*(rsdp_ptr as *const RSDPDescriptor2)),
            _=> panic!("invalid RSDP revision: {:#?}", buffer.unwrap())
        }*/
    }
}