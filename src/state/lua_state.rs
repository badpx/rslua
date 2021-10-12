use super::lua_stack::LuaStack;
use super::lua_value::LuaValue;
use crate::api::consts::*;
use crate::api::LuaAPI;

pub struct LuaState {
    pub stack: LuaStack,
}

impl LuaState {
    pub fn new() -> LuaState {
        LuaState {
            stack: LuaStack::new(20),
        }
    }

}