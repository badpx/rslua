use crate::number::math;
use crate::api::consts::*;

#[derive(Clone, PartialEq, Debug)]  // Add PartialEq & Debug for unit test.
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

    pub fn to_integer(&self) -> Option<i64> {
        match self {
            LuaValue::Integer(i) => Some(*i),
            LuaValue::Number(n) => math::float_to_integer(*n),
            LuaValue::Str(s) => string_to_integer(s),
            _ => None,
        }
    }

    pub fn to_number(&self) -> Option<f64> {
        match self {
            LuaValue::Integer(i) => Some(*i as f64),
            LuaValue::Number(n) => Some(*n),
            LuaValue::Str(s) => s.parse::<f64>().ok(),
            _ => None,
        }
    }
}

fn string_to_integer(s: &String) -> Option<i64> {
    if let Ok(i) = s.parse::<i64>() {
        Some(i)
    } else if let Ok(n) = s.parse::<f64>() {
        math::float_to_integer(n)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn lua_value2boolean() {
        assert_eq!(LuaValue::Nil.to_boolean(), false);
        assert_eq!(LuaValue::Boolean(true).to_boolean(), true);
        assert_eq!(LuaValue::Boolean(false).to_boolean(), false);
        assert_eq!(LuaValue::Integer(0).to_boolean(), true);
        assert_eq!(LuaValue::Integer(1).to_boolean(), true);
        assert_eq!(LuaValue::Number(0.0).to_boolean(), true);
        assert_eq!(LuaValue::Number(-1.1).to_boolean(), true);
        assert_eq!(LuaValue::Str("".to_string()).to_boolean(), true);
        assert_eq!(LuaValue::Str("false".to_string()).to_boolean(), true);
    }
    
    #[test]
    fn lua_value2integer() {
        assert_eq!(LuaValue::Integer(1024).to_integer(), Some(1024));
        assert_eq!(LuaValue::Integer(-1024).to_integer(), Some(-1024));
        assert_eq!(LuaValue::Number(0.99).to_integer(), None);
        assert_eq!(LuaValue::Number(99.0).to_integer(), Some(99));
        assert_eq!(LuaValue::Number(-0.99).to_integer(), None);
        assert_eq!(LuaValue::Number(-99.0).to_integer(), Some(-99));
        assert_eq!(LuaValue::Str("4096".to_string()).to_integer(), Some(4096));
        assert_eq!(LuaValue::Str("4096.00".to_string()).to_integer(), Some(4096));
        assert_eq!(LuaValue::Str("0.4096".to_string()).to_integer(), None);
        assert_eq!(LuaValue::Str("0xff".to_string()).to_integer(), None);
        assert_eq!(LuaValue::Str("010".to_string()).to_integer(), Some(10));
        assert_eq!(LuaValue::Str("0x10".to_string()).to_integer(), None);
        assert_eq!(LuaValue::Nil.to_integer(), None);
        assert_eq!(LuaValue::Boolean(true).to_integer(), None);
        assert_eq!(LuaValue::Boolean(false).to_integer(), None);
    }

    #[test]
    fn lua_value2number() {
        assert_eq!(LuaValue::Integer(1024).to_number(), Some(1024.0));
        assert_eq!(LuaValue::Integer(-1024).to_number(), Some(-1024.0));
        assert_eq!(LuaValue::Number(0.99).to_number(), Some(0.99));
        assert_eq!(LuaValue::Number(99.0).to_number(), Some(99.0));
        assert_eq!(LuaValue::Number(-0.99).to_number(), Some(-0.99));
        assert_eq!(LuaValue::Number(-99.0).to_number(), Some(-99.0));
        assert_eq!(LuaValue::Str("4096".to_string()).to_number(), Some(4096.0));
        assert_eq!(LuaValue::Str("4096.00".to_string()).to_number(), Some(4096.0));
        assert_eq!(LuaValue::Str("0.4096".to_string()).to_number(), Some(0.4096));
        assert_eq!(LuaValue::Str("0xff".to_string()).to_number(), None);
        assert_eq!(LuaValue::Str("010".to_string()).to_number(), Some(10.0));
        assert_eq!(LuaValue::Str("0x10".to_string()).to_number(), None);
        assert_eq!(LuaValue::Str(".01".to_string()).to_number(), Some(0.01));
        assert_eq!(LuaValue::Nil.to_number(), None);
        assert_eq!(LuaValue::Boolean(true).to_number(), None);
        assert_eq!(LuaValue::Boolean(false).to_number(), None);
    }
}