mod lua_value;
mod lua_stack;
mod lua_state;
mod lua_table;
mod api_stack;
mod api_arith;
mod api_compare;
mod closure;

pub use self::lua_state::LuaState;
use crate::binary::chunk::Prototype;
use std::rc::Rc;

pub fn new_lua_state(stack_size: usize, proto: Rc<Prototype>) -> LuaState {
    let mut ls = LuaState::new();
    ls.push_frame(self::lua_stack::LuaStack::new(stack_size, Rc::new(self::closure::Closure::new(proto))));
    ls
}
