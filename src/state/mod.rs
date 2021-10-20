mod api_arith;
mod api_compare;
mod api_stack;
mod closure;
mod lua_stack;
mod lua_state;
mod lua_table;
mod lua_value;

pub use self::lua_state::LuaState;
use crate::binary::chunk::Prototype;
use std::rc::Rc;

pub fn new_lua_state(stack_size: usize, proto: Rc<Prototype>) -> LuaState {
    let mut ls = LuaState::new();
    if let self::lua_value::LuaValue::Table(ref reg_table) = ls.registry {
        ls.push_frame(self::lua_stack::LuaStack::new(
                stack_size,
                Rc::new(self::closure::Closure::new_lua_closure(proto)),
                reg_table.clone()
        ));
        ls
    } else {
        panic!("Invalid registry!");
    }
}
