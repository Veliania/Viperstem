pub struct Stack<T> {
    values: [Option<T>; 128],
    current: usize
}

impl<T> Stack<T> where T: Copy + Clone {
    pub const fn new() -> Stack<T> {
        Stack { 
            values: [None; 128], 
            current: 0
        }
    }

    pub fn has_val(&self) -> bool  {
        match self.values[0] {
            Some(_val) => return true,
            _=> return false,
        }
    }

    pub fn push(&mut self, val: T) {
        self.values[self.current] = Some(val);
        self.current += 1;
    }

    pub fn pop(&mut self) -> T {
        self.current -= 1;
        let return_val = self.values[self.current].expect("Stack empty");

        self.values[self.current] = None;
        return_val
    }

    pub fn clear(&mut self) {
        for i in 0..self.current {
            self.values[i] = None;
        }

        self.current = 0;
    }
}