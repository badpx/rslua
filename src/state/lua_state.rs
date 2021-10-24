use super::closure::{Closure, UpValue};
use super::lua_stack::LuaStack;
use super::lua_table::LuaTable;
use super::lua_value::LuaValue;
use crate::api::consts::*;
use crate::api::LuaAPI;
use crate::api::LuaVM;
use crate::binary::chunk::Constant;
use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashMap;

pub struct LuaState {
    frames: Vec<LuaStack>,
    pub registry: LuaValue,
}

impl LuaState {
    pub fn new() -> LuaState {
        let tbl = Rc::new(RefCell::new(LuaTable::new(0, 0)));
        tbl.borrow_mut().put(
            LuaValue::Integer(LUA_RIDX_GLOBALS as i64),
            LuaValue::Table(Rc::new(RefCell::new(LuaTable::new(0, 0)))),
        ); // Global environment
        let dummy_closure = Rc::new(RefCell::new(Closure::new_dummy_closure()));
        let dummy_frame = LuaStack::new(LUA_MINSTACK, dummy_closure);
        LuaState {
            frames: vec![dummy_frame],
            registry: LuaValue::Table(tbl),
        }
    }

    pub fn stack_mut(&mut self) -> &mut LuaStack {
        self.frames.last_mut().unwrap()
    }

    pub fn stack(&self) -> &LuaStack {
        self.frames.last().unwrap()
    }

    pub fn push_frame(&mut self, frame: LuaStack) {
        self.frames.push(frame);
    }

    pub fn pop_frame(&mut self) -> LuaStack {
        self.frames.pop().unwrap()
    }
}

impl LuaVM for LuaState {
    fn pc(&self) -> isize {
        self.stack().pc
    }

    fn add_pc(&mut self, n: isize) {
        self.stack_mut().pc += n;
    }

    // Fetch next instruction
    fn fetch(&mut self) -> u32 {
        let inst = self.stack().closure.borrow().proto.code[self.stack().pc as usize];
        self.stack_mut().pc += 1;
        inst
    }

    fn get_const(&mut self, idx: isize) {
        let val = match &self.stack_mut().closure.borrow().proto.constants[idx as usize] {
            Constant::Nil => LuaValue::Nil,
            Constant::Boolean(b) => LuaValue::Boolean(*b),
            Constant::Integer(i) => LuaValue::Integer(*i),
            Constant::Number(n) => LuaValue::Number(*n),
            Constant::Str(s) => LuaValue::Str((*s).clone()),
        };
        self.stack_mut().push(val);
    }

    fn get_rk(&mut self, rk: isize) {
        if rk > 0xFF {
            // constant index
            self.get_const(rk & 0xFF);
        } else {
            // register index
            self.push_value(rk + 1);
        }
    }

    fn register_count(&self) -> usize {
        self.stack().closure.borrow().proto.max_stack_size as usize
    }

    fn load_vararg(&mut self, mut n: isize) {
        if n < 0 {
            n = self.stack().varargs.len() as isize;
        }
        let varargs = self.stack().varargs.clone();
        self.stack_mut().check(n as usize);
        self.stack_mut().push_n(varargs, n);
    }

    fn load_proto(&mut self, idx: usize) {
        let proto = self.stack().closure.borrow().proto.protos[idx].clone();
        let closure = LuaValue::new_lua_closure(proto.clone());

        if let LuaValue::Function(ref c) = closure {
            for (i, uv) in proto.upvalues.iter().enumerate() {
                let uv_idx = uv.idx as usize;
                if uv.instack == 1 {
                    match &self.stack().openuvs.get(&uv_idx) {
                        Some(openuv) => c.borrow_mut().upvals[i] = Some((*openuv).clone()),
                        None => {
                            let t = UpValue { val: Rc::new(RefCell::new(self.stack().get(uv_idx as isize))) };
                            c.borrow_mut().upvals[i] = Some(t.clone());
                            self.stack_mut().openuvs.insert(uv_idx, t);
                        },
                    }
                } else {
                    c.borrow_mut().upvals[i] = Some(self.stack().closure.borrow().upvals[uv_idx].as_ref().unwrap().clone());
                }
            }
        }

        // TODO: Need forward?
        self.stack_mut().push(closure);
    }
}
