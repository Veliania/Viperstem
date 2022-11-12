use crate::bitmap::BitMap;
//use crate::println;

#[derive(Copy, Clone)]
pub struct Sect {
    pub base: *mut u8,
    pub size: usize,
}

impl Sect {
    pub const fn null() -> Sect {
        Sect { 
            base: 0 as *mut u8, 
            size: 0
        }
    }

    pub fn new(size: usize, base: *mut u8) -> Sect {
        Sect {
            base,
            size
        }
    }
}

#[cfg(not(features = "extended-section-manager"))]
pub const SECTION_MAX: usize = 16;
#[cfg(features = "extended-section-manager")]
pub const SECTION_MAX: usize = 32;

pub struct SectManager {
    sections: [Option<Sect>; SECTION_MAX],
    bitmap_buf: [u8; SECTION_MAX / 8]
}

pub static mut SECTION_MANAGER: SectManager = SectManager::null();

impl SectManager {
    pub const fn null() -> SectManager {
        SectManager {
            sections: [None; SECTION_MAX],
            bitmap_buf: [0; SECTION_MAX / 8]
        }
    }

    pub fn add_sect(&mut self, new_sect: Sect) {
        for i in 0..SECTION_MAX {
            match self.sections[i] {
                None => {
                    self.sections[i] = Some(new_sect);
                    return
                }
                _ => ()
            }
        }

        panic!("Failed to add new section, not enough room, consider enabling extened-section-manager feature");
    }

    pub fn req_sect(&mut self) -> (Sect, usize) {
        let mybitmap = BitMap::new(SECTION_MAX / 8, &mut self.bitmap_buf[0]);

        for i in 0..self.sect_count() as usize {
            match mybitmap.get_bool(i) {
                true => (),
                false => {
                    match self.sections[i]{
                        None => (),
                        Some(val) => {
                            mybitmap.set_bool(i, true);
                            return (val, i);
                        }
                    }
                }
            }
        }

        panic!("No available sections");
    }

    pub fn req_large_sect(&mut self) -> (Sect, usize) {
        let mybitmap = BitMap::new(SECTION_MAX / 8, &mut self.bitmap_buf[0]);

        let mut biggest = (Sect::null(), 0);

        for i in 0..self.sect_count() as usize {
            match mybitmap.get_bool(i) {
                true => (),
                false => {
                    match self.sections[i]{
                        None => (),
                        Some(val) => {
                            if val.size > biggest.0.size {
                                biggest = (val, i);
                            }
                        }
                    }
                }
            }
        }

        mybitmap.set_bool(biggest.1, true);
        return (biggest.0, biggest.1);
    }

    pub fn req_sect_size(&mut self, size: usize) -> (Sect, usize) {
        let mybitmap = BitMap::new(SECTION_MAX / 8, &mut self.bitmap_buf[0]);

        let mut closest = (Sect::null(), 0, usize::MAX);

        for i in 0..self.sect_count() as usize {
            match mybitmap.get_bool(i) {
                true => (),
                false => {
                    match self.sections[i]{
                        None => (),
                        Some(val) => {
                            let difference = (val.size as isize) - (size as isize);

                            if difference == 0 {
                                mybitmap.set_bool(i, true);

                                return (val, i);
                            }

                            if difference > 0 {
                                let absolute = size.abs_diff(val.size);

                                if absolute < closest.2 {
                                    closest = (val, i, absolute);
                                }
                            }
                        }
                    }
                }
            }
        }

        mybitmap.set_bool(closest.1, true);
        return (closest.0, closest.1);
    }

    //unsafe because if called while section is in use, and then the section is requested, the section will be cleared
    pub unsafe fn ret_sect(&mut self, index: usize) {
        let mybitmap = BitMap::new(SECTION_MAX / 8, &mut self.bitmap_buf[0]);

        mybitmap.set_bool(index, false);
    }

    //unsafe because if called while section is in use, could delete important data
    pub unsafe fn empty_section(&mut self, index: usize) {
        let section = self.sections[index].expect("Section does not exist");

        for i in 0..section.size {
            *section.base.offset(i as isize) = 0;
        }
    }

    pub fn sect_count(&self) -> u8 {
        let mut total = 0;

        for i in 0..SECTION_MAX {
            match self.sections[i] {
                Some(_val) => total += 1,
                _ => ()
            }
        }

        total
    }

    pub fn space(&self) -> usize {
        let mut total = 0;

        for i in 0..SECTION_MAX {
            match self.sections[i] {
                Some(val) => total += val.size,
                _ => ()
            }
        }

        total
    }

    pub fn unused_sect(&mut self) -> u8 {
        let mybitmap = BitMap::new(SECTION_MAX / 8, &mut self.bitmap_buf[0]);
        let mut total = 0;

        for i in 0..sect_count() {
            match mybitmap.get_bool(i as usize) {
                true => (),
                false => total += 1,
            }
        }

        total
    }

    pub fn sect_size(&self, index: usize) -> usize {
        let section = self.sections[index];

        match section {
            None => panic!("Section does not exist"),
            Some(sect) => return sect.size
        }
    }

    pub fn is_used(&mut self, index: usize) -> bool {
        let mybitmap = BitMap::new(SECTION_MAX / 8, &mut self.bitmap_buf[0]);

        *mybitmap.get_bool(index)
    }
}

pub fn init_sect_manager() {
    let mmap = crate::MMAP
        .get_response()
        .get()
        .expect("barebones: recieved no mmap");

    unsafe {
        for entry in mmap.memmap() {
            //println!("{:?}", entry.as_ptr());
            match entry.typ {
                limine::LimineMemoryMapEntryType::Usable => SECTION_MANAGER.add_sect(Sect::new(entry.len as usize, entry.base as *mut u8)),
                _ => ()
            }
        }

        /*for entry_num in 0..mmap.entry_count {
            let entry = &*(&*mmap.entries.as_ptr()).as_ptr().offset((entry_num as isize) * 8);
            match entry.typ {
                limine::LimineMemoryMapEntryType::Usable => SECTION_MANAGER.add_sect(Sect::new(entry.len as usize, entry.base as *mut u8)),
                _ => ()
            }
        }*/
    }
}

pub fn req_sect() -> (Sect, usize) {
    unsafe {
        SECTION_MANAGER.req_sect()
    }
}

//requests a sector of a certain size, will give you a sector of the closest available size
pub fn req_sect_size(size: usize) -> (Sect, usize) {
    unsafe {
        SECTION_MANAGER.req_sect_size(size)
    }
}

pub fn req_large_sect() -> (Sect, usize) {
    unsafe {
        SECTION_MANAGER.req_large_sect()
    }
}

pub unsafe fn ret_sect(index: usize) {
    SECTION_MANAGER.ret_sect(index);
}

pub unsafe fn empty_section(index: usize) {
    SECTION_MANAGER.empty_section(index);
}

pub fn sect_count() -> u8 {
    unsafe {
        return SECTION_MANAGER.sect_count();
    }
}

pub fn space() -> usize {
    unsafe {
        return SECTION_MANAGER.space();
    }
}

pub fn unused_sect() -> u8 {
    unsafe {
        return SECTION_MANAGER.unused_sect();
    }
}

pub fn sect_size(index: usize) -> usize {
    unsafe {
        SECTION_MANAGER.sect_size(index)
    }
}

pub fn is_used(index: usize) -> bool {
    unsafe {
        SECTION_MANAGER.is_used(index)
    }
}