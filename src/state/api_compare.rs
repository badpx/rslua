use super::lua_value::LuaValue;
use crate::api::consts::*;

pub fn compare(a: &LuaValue, b: &LuaValue, op: CompareOp) -> bool {
    match op {
        LUA_OPEQ => _eq(a, b),
        LUA_OPLT => _lt(a, b),
        LUA_OPLE => _le(a, b),
        _ => false,
    }
}

macro_rules! cmp {
    ($a: ident $op: tt $b:ident) => {
        match $a {
            LuaValue::Integer(x) => match $b {
                LuaValue::Integer(y) => x $op y,
                LuaValue::Number(y) => (*x as f64) $op *y,
                _ => false,
            },
            LuaValue::Number(x) => match $b {
                LuaValue::Integer(y) => *x $op (*y as f64),
                LuaValue::Number(y) => x $op y,
                _ => false,
            },
            LuaValue::Str(x) => match $b {
                LuaValue::Str(y) => x $op y,
                _ => false,
            }
            _ => false,
        }
    };
}

fn _eq(a: &LuaValue, b: &LuaValue) -> bool {
    match a {
        LuaValue::Nil => match b {
            LuaValue::Nil => true,
            _ => false,
        },
        LuaValue::Boolean(x) => match b {
            LuaValue::Boolean(y) => x == y,
            _ => false,
        },
        _ => {
            cmp!(a == b)
        },
    }
}

fn _lt(a: &LuaValue, b: &LuaValue) -> bool {
    cmp!(a < b)
}

fn _le(a: &LuaValue, b: &LuaValue) -> bool {
    cmp!(a <= b)
}