use crate::api::LuaVM;
use crate::binary::chunk::{Constant, Prototype};
use super::lua_stack::LuaStack;
use super::lua_value::LuaValue;
use crate::api::LuaAPI;
use super::closure::Closure;
use std::rc::Rc;


pub struct LuaState {
    frames: Vec<LuaStack>,
}

impl LuaState {
    pub fn new(stack_size: usize, proto: Prototype) -> LuaState {
        let closure = Rc::new(Closure::new(Rc::new(proto)));
        let frame = LuaStack::new(stack_size, closure);

        LuaState {
            frames: vec![frame],
        }
    }

    pub fn stack_mut(&mut self) -> &mut LuaStack {
        self.frames.last_mut().unwrap()
    }

    pub fn stack(&self) -> &LuaStack {
        self.frames.last().unwrap()
    }

    fn push_frame(&mut self, frame: LuaStack) {
        self.frames.push(frame);
    }

    fn pop_frame(&mut self) -> LuaStack {
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

}