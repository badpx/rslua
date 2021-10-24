use std::cell::RefCell;
use std::rc::Rc;
use std::fmt;
use std::hash::Hash;
use crate::number::math;
use crate::api::consts::*;
use crate::binary::chunk::Prototype;
use super::lua_table::LuaTable;
use super::closure::Closure;

#[derive(Clone)]  // Add PartialEq & Debug for unit test.
pub enum LuaValue {
    Nil,
    Boolean(bool),
    Number(f64),
    Integer(i64),
    Str(String),
    Table(Rc<RefCell<LuaTable>>),   // mutability inside of something immutable.
    Function(Rc<RefCell<Closure>>),
}

impl fmt::Debug for LuaValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LuaValue::Nil => write!(f, "(nil)"),
            LuaValue::Boolean(b) => write!(f, "({})", b),
            LuaValue::Integer(i) => write!(f, "({})", i),
            LuaValue::Number(n) => write!(f, "({})", n),
            LuaValue::Str(s) => write!(f, "(\"{}\")", s),
            LuaValue::Table(_) => write!(f, "(table)"),
            LuaValue::Function(_) => write!(f, "(function)"),
        }
    }
}

impl PartialEq for LuaValue {
    fn eq(&self, other: &LuaValue) -> bool {
        if let (LuaValue::Nil, LuaValue::Nil) = (self, other) {
            true
        } else if let (LuaValue::Boolean(x), LuaValue::Boolean(y)) = (self, other) {
            x == y
        } else if let (LuaValue::Integer(x), LuaValue::Integer(y)) = (self, other) {
            x == y
        } else if let (LuaValue::Number(x), LuaValue::Number(y)) = (self, other) {
            x == y
        }  else if let (LuaValue::Str(x), LuaValue::Str(y)) = (self, other) {
            x == y
        }  else if let (LuaValue::Table(x), LuaValue::Table(y)) = (self, other) {
            Rc::ptr_eq(x, y)
        }  else if let (LuaValue::Function(x), LuaValue::Function(y)) = (self, other) {
            Rc::ptr_eq(x, y)
        } else {
            false
        }
    }
}

// the trait `std::cmp::Eq` is not implemented for `f64`
impl Eq for LuaValue { }

// the trait `std::hash::Hash` is not implemented for `f64`
impl Hash for LuaValue {
    fn hash<H>(&self, state: &mut H) where H: std::hash::Hasher {
        match self {
            LuaValue::Nil => 0.hash(state),
            LuaValue::Boolean(b) => b.hash(state),
            LuaValue::Integer(i) => i.hash(state),
            LuaValue::Number(n) => n.to_bits().hash(state),
            LuaValue::Str(s) => s.hash(state),
            LuaValue::Table(t) => t.borrow().hash(state),
            LuaValue::Function(f) => f.borrow().hash(state),
        }
    }
}

impl LuaValue {
    pub fn new_table(narr: usize, nrec: usize) -> LuaValue {
        LuaValue::Table(Rc::new(RefCell::new(LuaTable::new(narr, nrec))))
    }

    pub fn new_lua_closure(proto: Rc<Prototype>) -> LuaValue {
        LuaValue::Function(Rc::new(RefCell::new(Closure::new_lua_closure(proto))))
    }

    pub fn is_nil(&self) -> bool {
        match self {
            LuaValue::Nil => true,
            _ => false,
        }
    }
    pub fn type_id(&self) -> LuaType {
        match self {
            LuaValue::Nil => LUA_TNIL,
            LuaValue::Boolean(_) => LUA_TBOOLEAN,
            LuaValue::Integer(_) | LuaValue::Number(_) => LUA_TNUMBER,
            LuaValue::Str(_) => LUA_TSTRING,
            LuaValue::Table(_) => LUA_TTABLE,
            LuaValue::Function(_) => LUA_TFUNCTION,
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
