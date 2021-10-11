mod lua_value;
mod lua_stack;
mod lua_state;
mod api_stack;

pub use self::lua_state::LuaState;

pub fn new_lua_state() -> LuaState {
    LuaState::new()
}