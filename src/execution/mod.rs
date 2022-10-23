#[derive(Clone, Copy)]
pub struct FuncEntry {
    ptr: Option<fn()>,
    //next: Option<u8>
}

impl FuncEntry {
    pub const fn null() -> FuncEntry {
        return FuncEntry { ptr: None/*, next: None*/ };
    }

    pub fn new(data: fn()/*, next: u8*/) -> FuncEntry {
        FuncEntry { ptr: Some(data)/*, next: Some(next)*/ }
    }
}

pub struct FuncStack {
    stack: [FuncEntry; 256],
    current: u8
}

impl FuncStack {
    fn push(&mut self, data: fn()) {
        match self.is_exec() {
            true => {
                for i in self.current as usize..256 {
                    if (self.stack[i].ptr == None) && (self.stack[i - 1].ptr != None) {
                        self.stack[i] = FuncEntry::new(data/*, (i + 1) as u8*/);
                    }
                }
            }, 
            false => self.stack[0] = FuncEntry::new(data/*, 1*/)
        }
    }

    fn pop(&mut self) -> Option<fn()> {
        for i in 0..256 {
            if self.stack[i].ptr != None {
                let ret = self.stack[i].ptr;
                self.stack[i].ptr = None;

                return ret;
            }
        }

        return None;
    }

    fn is_exec(&self) -> bool {
        for i in 0..256 {
            if self.stack[i].ptr != None {
                return true;
            }
        }

        return false;
    }
}

pub static mut EXEC_LOOP: FuncStack = FuncStack {
    stack: [FuncEntry::null(); 256],
    current: 0
};

pub fn kern_exec() {
    unsafe {
        match EXEC_LOOP.is_exec() {
            true => EXEC_LOOP.pop().expect("Error finding function in execution loop")(),
            false => x86_64::instructions::hlt()
        }
    }
}

pub fn kern_push(data: fn()) {
    unsafe {
        EXEC_LOOP.push(data);
    }
}