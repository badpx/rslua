mod lua_value;
mod lua_stack;
mod lua_state;
mod api_stack;
mod api_arith;
mod api_compare;

pub use self::lua_state::LuaState;
use crate::binary::chunk::Prototype;

pub fn new_lua_state(stack_size: usize, proto: Prototype) -> LuaState {
    LuaState::new(stack_size, proto)
}
