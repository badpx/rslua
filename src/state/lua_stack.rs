use std::rc::Rc;
use core::cell::RefCell;
use super::lua_value::LuaValue;
use super::closure::Closure;
use super::lua_table::LuaTable;

pub struct LuaStack {
    slots: Vec<LuaValue>,
    pub closure: Rc<Closure>,
    pub varargs: Vec<LuaValue>,
    pub pc: isize,
    reg_table: Rc<RefCell<LuaTable>>,
}

impl LuaStack {
    pub fn new(size: usize, closure: Rc<Closure>, reg_table: Rc<RefCell<LuaTable>>) -> LuaStack {
        return LuaStack {
            slots: Vec::with_capacity(size),
            closure,
            varargs: Vec::new(),
            pc: 0,
            reg_table,
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

    pub fn push_n(&mut self, mut vals: Vec<LuaValue>, n: isize) {
        vals.reverse();
        let nvals = vals.len();
        let un = if n < 0 { nvals } else { n as usize };

        for i in 0..un {
            if i < nvals {
                self.push(vals.pop().unwrap());
            } else {
                self.push(LuaValue::Nil);
            }
        }
    }

    pub fn pop(&mut self) -> LuaValue {
        self.slots.pop().unwrap()
    }

    pub fn pop_n(&mut self, n: usize) -> Vec<LuaValue> {
        let mut vec = Vec::with_capacity(n);
        for _ in 0..n {
            vec.push(self.pop());
        }
        vec.reverse();
        vec
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

    pub fn set_top(&mut self, idx: isize) {
        let new_top = self.abs_index(idx);
        if new_top < 0 {
            panic!("Stack underflow!");
        }
        let n = self.top() - new_top;
        if n > 0 {
            for _ in 0..n {
                self.pop();
            }
        } else {
            for _ in n..0 {
                self.push(LuaValue::Nil);
            }
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
