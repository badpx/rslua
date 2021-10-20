use super::closure::Closure;
use super::lua_stack::LuaStack;
use super::lua_table::LuaTable;
use super::lua_value::LuaValue;
use crate::api::consts::*;
use crate::api::LuaAPI;
use crate::api::LuaVM;
use crate::binary::chunk::{Constant, Prototype};
use core::cell::RefCell;
use std::rc::Rc;

pub struct LuaState {
    frames: Vec<LuaStack>,
    pub registry: LuaValue,
}

impl LuaState {
    pub fn new() -> LuaState {
        let reg_table = Rc::new(RefCell::new(LuaTable::new(0, 0)));
        reg_table.borrow_mut().put(
            LuaValue::Integer(LUA_RIDX_GLOBALS as i64),
            LuaValue::Table(Rc::new(RefCell::new(LuaTable::new(0, 0)))),
        ); // Global environment
        let dummy_closure = Rc::new(Closure::new_dummy_closure());
        let dummy_frame = LuaStack::new(LUA_MINSTACK, dummy_closure, reg_table.clone());
        LuaState {
            frames: vec![dummy_frame],
            registry: LuaValue::Table(reg_table),
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
        let inst = self.stack().closure.proto.code[self.stack().pc as usize];
        self.stack_mut().pc += 1;
        inst
    }

    fn get_const(&mut self, idx: isize) {
        let c = &self.stack().closure.proto.constants[idx as usize];
        let val = match c {
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
        self.stack().closure.proto.max_stack_size as usize
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
        let proto = self.stack().closure.proto.protos[idx].clone();
        let closure = LuaValue::new_lua_closure(proto);
        self.stack_mut().push(closure);
    }
}
