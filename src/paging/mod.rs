use core::intrinsics::size_of;

use x86_64::{structures::paging::{PageTable, PageTableFlags, PhysFrame, page_table::PageTableEntry, Page, page}, PhysAddr, registers::control::*};
use crate::{*, output::FRAME_REQUEST, bitmap::BitMap};

static mut _PAGE_FRAME: Option<u64> = None;
static mut PML4: PageTable = PageTable::new();
static mut is_used_bitmap: [u8; 32_768] = [0; 32_768];

pub fn paging() {
    let new_page_frame = req_sect_size(258 * size_of::<PageTable>());

    println!("requested size: {}", 4 * size_of::<PageTable>());

    println!("page frame: {:?}\nsize: {}", new_page_frame.0.base, new_page_frame.0.size);

    let base_addr = new_page_frame.0.base as *mut PageTable;

    unsafe {
        let current_flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

        let PDP_ptr = base_addr;
        *PDP_ptr = PageTable::new();

        let PD_ptr = base_addr.offset(size_of::<PageTable>() as isize);
        *PD_ptr = PageTable::new();

        let PT_ptr = base_addr.offset(2 * size_of::<PageTable>() as isize) as *mut [PageTable; 256];
        for mut entry in (*PT_ptr).iter_mut() {
            entry = &mut PageTable::new();
        }

        let old_pml4 = Cr3::read().0.start_address().as_u64() as *mut PageTable;

        println!("Getting frame buffer virt addr");
        //let framebuffer_virt = VirtAddr::new_truncate(FRAME_REQUEST.get_response().get().unwrap().framebuffers()[0].address.as_ptr().unwrap() as u64);
        //let framebuffer_addr = virt_to_phys(framebuffer_virt, old_pml4).expect("Could not find virtual address in page map");

        
        let mut wrapped_framebuffer_addr: Option<PhysAddr> = None;
        for entry in MMAP.get_response().get().unwrap().memmap() {
            match entry.typ {
                LimineMemoryMapEntryType::Framebuffer => {
                    wrapped_framebuffer_addr = Some(PhysAddr::new_truncate(entry.base));
                    break
                }
                _ => {}
            }
        }

        let framebuffer_addr = match wrapped_framebuffer_addr {
            Some(val) => val,
            None => panic!("Could not find framebuffer")
        };

        PML4[0].set_addr(PhysAddr::new_truncate(PDP_ptr as u64), current_flags);
        (*PDP_ptr)[0].set_addr(PhysAddr::new_truncate(PD_ptr as u64), current_flags);
        //(*PD_ptr)[0].set_addr(PhysAddr::new_truncate(PT_ptr as u64), current_flags);
        for i in 0..256 {
            (*PD_ptr)[i].set_addr(PhysAddr::new_truncate((&mut (*PT_ptr)) as *mut PageTable as u64), current_flags);
        }


        let flags = Cr3::read().1;
        let frame = PhysFrame::from_start_address(PhysAddr::new_truncate((&mut PML4 as *mut PageTable) as u64)).expect("Page map level 4 could not be loaded as a physframe");
        Cr3::write(frame, flags);
        
        println!("Mapping frame buffer");
        let frameflags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE;
        map_section(framebuffer_addr, VirtAddr::new_truncate(0), frameflags);
        
        output::init_post_paging(VirtAddr::new_truncate(0)); //MUST RUN IMMEDIATELY AFTER PAGING, OTHERWISE OUTPUT WONT WORK
    }
}

pub unsafe fn map_section(addr: PhysAddr, virt_addr: VirtAddr, flags: PageTableFlags) {
    let pml4_entry_bits = (virt_addr.as_u64() >> 39) & 0b111111111;
    let pdp_entry_bits = (virt_addr.as_u64() >> 30) & 0b111111111;
    let pd_entry_bits = (virt_addr.as_u64() >> 21) & 0b111111111;
    let pt_entry_bits = (virt_addr.as_u64() >> 12) & 0b111111111;

    let pdp = PML4[pml4_entry_bits as usize].addr().as_u64() as *mut PageTable;
    let pd = (*pdp)[pdp_entry_bits as usize].addr().as_u64() as *mut PageTable;
    let pt = (*pd)[pd_entry_bits as usize].addr().as_u64() as *mut PageTable;
    
    println!("Setting address for page");
    (*pt)[pt_entry_bits as usize].set_addr(addr, flags);
    println!("Address for page set");
}

pub fn virt_to_phys(addr: VirtAddr, pml4: *mut PageTable) -> Option<PhysAddr> {
    let pml4_entry_bits = (addr.as_u64() >> 39) & 0b111111111;
    let pdp_entry_bits = (addr.as_u64() >> 30) & 0b111111111;
    let pd_entry_bits = (addr.as_u64() >> 21) & 0b111111111;
    let pt_entry_bits = (addr.as_u64() >> 12) & 0b111111111;

    unsafe {
        if ((*pml4)[pml4_entry_bits as usize].flags() & PageTableFlags::PRESENT) != PageTableFlags::PRESENT {
            return None;
        }

        let pdp = PML4[pml4_entry_bits as usize].addr().as_u64() as *mut PageTable;
        if ((*pdp)[pdp_entry_bits as usize].flags() & PageTableFlags::PRESENT) != PageTableFlags::PRESENT {
            return None;
        }

        let pd = (*pdp)[pdp_entry_bits as usize].addr().as_u64() as *mut PageTable;
        if ((*pd)[pd_entry_bits as usize].flags() & PageTableFlags::PRESENT) != PageTableFlags::PRESENT {
            return None;
        }

        let pt = (*pd)[pd_entry_bits as usize].addr().as_u64() as *mut PageTable;
        if ((*pt)[pt_entry_bits as usize].flags() & PageTableFlags::PRESENT) != PageTableFlags::PRESENT {
            return None;
        }

        return Some((*pt)[pt_entry_bits as usize].addr());
    }
}

pub fn get_free_mem_size() -> (u128, u64) {
    let mmap = MMAP
        .get_response()
        .get()
        .expect("barebones: recieved no mmap");

    let mut total_sections: u64 = 0;
    let mut total = 0;


    for entry_num in 0..mmap.entry_count {
        unsafe {
            let entry = &*(&*mmap.entries.as_ptr()).as_ptr().offset((entry_num as isize) * size_of::<*mut LimineMemmapEntry>() as isize);
            match entry.typ {
                LimineMemoryMapEntryType::Usable => {
                    total += entry.len as u128;
                    total_sections += 1;
                },
                _ => ()
            }
        }
    }

    (total, total_sections)
}