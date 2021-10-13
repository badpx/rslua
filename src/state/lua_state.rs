use crate::api::LuaVM;
use crate::binary::chunk::{Constant, Prototype};
use super::lua_stack::LuaStack;
use super::lua_value::LuaValue;
use crate::api::LuaAPI;


pub struct LuaState {
    pub stack: LuaStack,
    pub proto: Prototype,
    pub pc: isize,
}

impl LuaState {
    pub fn new(stack_size: usize, proto: Prototype) -> LuaState {
        LuaState {
            stack: LuaStack::new(stack_size),
            proto,
            pc: 0,
        }
    }
}

impl LuaVM for LuaState {
fn pc(&self) -> isize {
    self.pc
}

fn add_pc(&mut self, n: isize) {
    self.pc += n;
}

// Fetch next instruction
fn fetch(&mut self) -> u32 {
    let inst = self.proto.code[self.pc as usize];
    self.pc += 1;
    inst
}

fn get_const(&mut self, idx: isize) {
    let c = &self.proto.constants[idx as usize];
    let val = match c {
        Constant::Nil => LuaValue::Nil,
        Constant::Boolean(b) => LuaValue::Boolean(*b),
        Constant::Integer(i) => LuaValue::Integer(*i),
        Constant::Number(n) => LuaValue::Number(*n),
        Constant::String(s) => LuaValue::Str(s.clone()),
    };
    self.stack.push(val);
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

}