use crate::api::consts::*;

#[derive(Clone)]
pub enum LuaValue {
    Nil,
    Boolean(bool),
    Number(f64),
    Integer(i64),
    Str(String),
}

impl LuaValue {
    pub fn type_id(&self) -> LuaType {
        match self {
            LuaValue::Nil => LUA_TNIL,
            LuaValue::Boolean(_) => LUA_TBOOLEAN,
            LuaValue::Integer(_) | LuaValue::Number(_) => LUA_TNUMBER,
            LuaValue::Str(_) => LUA_TSTRING,
        }
    }

    pub fn to_boolean(&self) -> bool {
        match self {
            LuaValue::Nil => false,
            LuaValue::Boolean(b) => *b,
            _ => true,
        }
    }

}