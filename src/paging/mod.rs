use core::{intrinsics::size_of, ops::Add};

use x86_64::{structures::paging::{PageTable, PageTableFlags, PhysFrame, page_table::PageTableEntry, Page}, PhysAddr, registers::control::*};
use crate::*;

static mut _PAGE_FRAME: Option<u64> = None;
static mut PML4: PageTable = PageTable::new();

pub fn paging() {
    let new_page_frame = req_sect_size(514 * size_of::<PageTable>());

    println!("requested size: {}", 514 * size_of::<PageTable>());

    println!("page frame: {:?}\nsize: {}", new_page_frame.0.base, new_page_frame.0.size);

    let base_addr = new_page_frame.0.base as *mut PageTable;

    unsafe {
        let current_flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

        let PDP_ptr = base_addr as *mut [PageTable; 2];
        for i in 0..2 {
            core::ptr::write(PDP_ptr.byte_add(i * 8) as *mut PageTable, PageTable::new());
        }
        println!("PD pointer: {:?}", PDP_ptr);

        let PD_ptr = (PDP_ptr.offset(1)) as *mut [PageTable; 512];
        for i in 0..512 {
            core::ptr::write(PD_ptr.byte_add(i * 8) as *mut PageTable, PageTable::new());
        }
        println!("PD pointer: {:?}", PD_ptr);

        let PT_ptr = (PD_ptr.offset(1)) as *mut [[PageTable; 512]; 512];
        for x in 0..512 {
            for y in 0..512 {
                core::ptr::write(PT_ptr.byte_add((y * 8) + (x * 4096)) as *mut PageTable, PageTable::new());
            }
        }
        println!("PT pointer: {:?}", PT_ptr);


        println!("Getting frame buffer virt addr");        
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
            //None => panic!("Could not find framebuffer")
            None => PhysAddr::new(output::ADDRESS.lock().unwrap().as_u64())
        };

        PML4[511].set_addr(PhysAddr::new_truncate(PDP_ptr as u64), current_flags);
        (*PDP_ptr)[0][510].set_addr(PhysAddr::new_truncate(PD_ptr as u64), current_flags);
        (*PDP_ptr)[0][511].set_addr(PhysAddr::new_truncate(PD_ptr.byte_add(size_of::<PageTable>()) as u64), current_flags);
        for y in 0..512 {
            (*PDP_ptr)[1][y].set_addr(PhysAddr::new_truncate((&mut (*PD_ptr)[y] as *mut PageTable) as u64), current_flags);
        }

        for x in 0..512 {
            for y in 0..512 {
                (*PD_ptr)[x][y].set_addr(PhysAddr::new_truncate((&mut (*PT_ptr)[x][y] as *mut PageTable) as u64), current_flags);
            }
        }
        //(*PD_ptr)[0].set_addr(PhysAddr::new_truncate(PT_ptr as u64), current_flags);

        println!("Mapping frame buffer");
        let frameflags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE;
        map_section(framebuffer_addr, VirtAddr::new_truncate(0xffffffff80000000), frameflags);
        map_section(framebuffer_addr.add(0x1000 as usize), VirtAddr::new_truncate(0xffffffff80001000), frameflags);
        map_section(framebuffer_addr.add(0x2000 as usize), VirtAddr::new_truncate(0xffffffff80002000), frameflags);
        map_section(framebuffer_addr.add(0x3000 as usize), VirtAddr::new_truncate(0xffffffff80003000), frameflags);

        map_kernel(); //maps kernel so we can still use the code
        let flags = Cr3::read().1;
        let frame = PhysFrame::from_start_address(PhysAddr::new_truncate((&mut PML4 as *mut PageTable) as u64)).expect("Page map level 4 could not be loaded as a physframe");
        println!("Writing to Cr3");
        Cr3::write(frame, flags);
        
        core::arch::asm!("int 0x1"); //Use debug interrupt to check if code after Cr3 write runs

        output::init_post_paging(VirtAddr::new_truncate(0xffffffff80000000)); //MUST RUN IMMEDIATELY AFTER PAGING, OTHERWISE OUTPUT WONT WORK
    }
}

pub fn map_kernel() {
    let mut kernel = Kernel::new();
    kernel.get();

    for i in 0..kernel.len / 0x1000 {
        unsafe {
            let phys = kernel.phys + (i * 0x1000);
            let virt = kernel.virt + (i * 0x1000);
            let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::GLOBAL;
            map_section(phys, virt, flags);
        }
    }
}

pub unsafe fn map_section(addr: PhysAddr, virt_addr: VirtAddr, flags: PageTableFlags) {
    let pml4_entry_bits = (virt_addr.as_u64() >> 39) & 0b111111111;
    let pdp_entry_bits = (virt_addr.as_u64() >> 30) & 0b111111111;
    let pd_entry_bits = (virt_addr.as_u64() >> 21) & 0b111111111;
    let pt_entry_bits = (virt_addr.as_u64() >> 12) & 0b111111111;

    assert!((&mut PML4 as *mut PageTable) != 0 as *mut PageTable, "PML4 pointer was null");
    
    let pdp = PML4[pml4_entry_bits as usize].addr().as_u64() as *mut PageTable;
    assert!(PML4[pml4_entry_bits as usize].flags() & PageTableFlags::PRESENT == PageTableFlags::PRESENT, "No present entry at {} in PML4", pml4_entry_bits);
    assert!(pdp != 0 as *mut PageTable, "PDP pointer was null, {:?}", pdp);
    //println!("PDP pointer: {:?}", pdp);

    let pd = (*pdp)[pdp_entry_bits as usize].addr().as_u64() as *mut PageTable;
    assert!((*pdp)[pdp_entry_bits as usize].flags() & PageTableFlags::PRESENT == PageTableFlags::PRESENT, "No present entry at {} in PDP", pdp_entry_bits);
    assert!(pd != 0 as *mut PageTable, "PD pointer was null, {:?}", pd);
    //println!("PD pointer: {:?}", pd);

    let pt = (*pd)[pd_entry_bits as usize].addr().as_u64() as *mut PageTable;
    //println!("PT pointer: {:?}", pt);
    //println!("PD[{}]: {:?}", pd_entry_bits, (*pd)[pd_entry_bits as usize].addr().as_u64() as *mut PageTable);

    assert!((*pd)[pd_entry_bits as usize].flags() & PageTableFlags::PRESENT == PageTableFlags::PRESENT, "No present entry at {} in PD", pd_entry_bits);
    assert!(pt != 0 as *mut PageTable, "PT pointer was null, {:?} entry number in pd: {}", pt, pd_entry_bits);
    
    let full_flags = flags | PageTableFlags::PRESENT;

    //println!("Mapping 0x{:x} to 0x{:x}", addr.as_u64(), virt_addr.as_u64());

    //println!("Setting address for page");
    (*pt).iter_mut().nth(pt_entry_bits as usize).expect("Could not find entry").set_addr(addr, full_flags);
    //(*pt)[pt_entry_bits as usize].set_addr(addr, flags);
    //println!("Address for page set");
}

pub fn virt_to_phys(addr: VirtAddr, pml4: *mut PageTable) -> Option<PhysAddr> {
    let pml4_entry_bits = (addr.as_u64() >> 39) & 0b111111111;
    let pdp_entry_bits = (addr.as_u64() >> 30) & 0b111111111;
    let pd_entry_bits = (addr.as_u64() >> 21) & 0b111111111;
    let pt_entry_bits = (addr.as_u64() >> 12) & 0b111111111;

    unsafe {
        if !(*pml4)[pml4_entry_bits as usize].is_present() {
            return None;
        }

        let pdp = PML4[pml4_entry_bits as usize].addr().as_u64() as *mut PageTable;
        if !(*pdp)[pdp_entry_bits as usize].is_present() {
            return None;
        }

        let pd = (*pdp)[pdp_entry_bits as usize].addr().as_u64() as *mut PageTable;
        if (*pd)[pd_entry_bits as usize].is_present() {
            return None;
        }

        let pt = (*pd)[pd_entry_bits as usize].addr().as_u64() as *mut PageTable;
        if !(*pt)[pt_entry_bits as usize].is_present() {
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

struct Kernel {
    phys: PhysAddr,
    virt: VirtAddr,
    len: usize
}

impl Kernel {
    pub fn new() -> Kernel {
        Kernel { phys: PhysAddr::new(0), virt: VirtAddr::new(0), len: 0 }
    }

    pub fn get(&mut self) {
        let kernel_address_response = KERN_POS.get_response().get().expect("Could not get Kernel address");
        for entry in MMAP.get_response().get().unwrap().memmap() {
            match entry.typ {
                LimineMemoryMapEntryType::KernelAndModules => self.len = entry.len as usize,
                _ => ()
            }
        }
        
        self.phys = PhysAddr::new(kernel_address_response.physical_base);
        self.virt = VirtAddr::new(kernel_address_response.virtual_base);
    }
}

trait IsPresent {
    fn is_present(&mut self) -> bool;
}

impl IsPresent for PageTableEntry {
    fn is_present(&mut self) -> bool {
        return (self.flags() & PageTableFlags::PRESENT) == PageTableFlags::PRESENT;
    }
}