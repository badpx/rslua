use super::lua_value::LuaValue;

pub struct LuaStack {
    slots: Vec<LuaValue>,
}

impl LuaStack {
    pub fn new(size: usize) -> LuaStack {
        return LuaStack {
            slots: Vec::with_capacity(size),
        }
    }

    pub fn check(&mut self, n: usize) {
        self.slots.reserve(n);
    }

    pub fn top(&self) -> isize {
        self.slots.len() as isize
    }

    pub fn push(&mut self, val: LuaValue) {
        self.slots.push(val);
    }

    pub fn pop(&mut self) -> LuaValue {
        self.slots.pop().unwrap()
    }

    pub fn abs_index(&self, idx: isize) -> isize {
        if idx >= 0 {
            idx
        } else {
            idx + self.top() + 1
        }
    }

    pub fn is_valid(&self, idx: isize) -> bool {
        self._is_valid(idx).0
    }

    fn _is_valid(&self, idx: isize) -> (bool, isize) {
        let abs_idx = self.abs_index(idx);
        (abs_idx > 0 && abs_idx <= self.top(), abs_idx)
    }

    pub fn get(&self, idx: isize) -> LuaValue {
        let (valid, abs_idx) = self._is_valid(idx);
        if valid {
            self.slots[abs_idx as usize - 1].clone()
        } else {
            LuaValue::Nil
        }
    }

    pub fn set(&mut self, idx: isize, val: LuaValue) {
        let (valid, abs_idx) = self._is_valid(idx);
        if valid {
            self.slots[abs_idx as usize - 1] = val;
        } else {
            panic!("Invalid index!");
        }
    }

    pub fn reverse(&mut self, mut from: usize, mut to: usize) {
        while from < to {
            self.slots.swap(from, to);
            from += 1;
            to -= 1;
        }
    }

    pub fn _raw_data(&self) -> &Vec<LuaValue> {
        &self.slots 
    }
}
