use crate::api::LuaAPI;
use super::lua_state::LuaState;
use crate::api::consts::*;
use super::lua_value::LuaValue;

impl LuaAPI for LuaState {
    /* =========================== Basic Methods =========================== */
    fn get_top(&self) -> isize {
        self.stack.top()
    }

    fn abs_index(&self, idx: isize) -> isize {
        self.stack.abs_index(idx)
    }

    fn check_stack(&mut self, n: usize) -> bool {
        self.stack.check(n);
        return true;    // Never fails
    }

    /*
                  pop(2)
        +-------+       +-------+
        |   d   |       |       |
        +-------+       +-------+
        |   c   |       |       |
        +-------+       +-------+
        |   b   |       |   b   |
        +-------+       +-------+
        |   a   | ====> |   a   |
        +-------+       +-------+
    */
    fn pop(&mut self, n: usize) {
        self.set_top(-(n as isize) - 1);
    }

    /*
                copy(2,3)
        +-------+       +-------+
        |       |       |       |
        +-------+       +-------+
        |   c   |   +-->|   b   |
        +-------+   |   +-------+
        |   b   |---+   |   b   |
        +-------+       +-------+
        |   a   |       |   a   |
        +-------+       +-------+
    */
    fn copy(&mut self, from: isize, to: isize) {
        let val = self.stack.get(from);
        self.stack.set(to, val);
    }

    /*
                 push(2)
        +-------+       +-------+
        |       |   +-->|   b   |
        +-------+   |   +-------+
        |   c   |   |   |   c   |
        +-------+   |   +-------+
        |   b   |---+   |   b   |
        +-------+       +-------+
        |   a   |       |   a   |
        +-------+       +-------+
    */
    fn push_value(&mut self, idx: isize) {
        let val = self.stack.get(idx);
        self.stack.push(val);
    }

    /*
                replace(2)
        +-------+       +-------+
        |   d   |---+   |       |
        +-------+   |   +-------+
        |   c   |   |   |   c   |
        +-------+   |   +-------+
        |   b   |   +-->|   d   |
        +-------+       +-------+
        |   a   |       |   a   |
        +-------+       +-------+
    */
    fn replace(&mut self, idx: isize) {
        let val = self.stack.pop();
        self.stack.set(idx, val);
    }

    /*
                insert(2)
        +-------+        +-------+
        |   d   |---+/-->|   c   |
        +-------+  /|    +-------+
        |   c   |/  |/-->|   b   |
        +-------+  /|    +-------+
        |   b   |/  +--->|   d   |
        +-------+        +-------+
        |   a   |        |   a   |
        +-------+        +-------+
    */
    fn insert(&mut self, idx: isize) {
        self.rotate(idx, 1);
    }

    /*
                 remove(2)
        +-------+        +-------+
        |   d   |---+    |       |
        +-------+    \   +-------+
        |   c   |--+  +->|   d   |
        +-------+   \    +-------+
        |   b   |->X +-->|   c   |
        +-------+        +-------+
        |   a   |        |   a   |
        +-------+        +-------+
    */
    fn remove(&mut self, idx: isize) {
        self.rotate(idx, -1);
        self.pop(1);
    }

    fn rotate(&mut self, idx: isize, n: isize) {
        let t = self.stack.top() - 1;
        let p = self.stack.abs_index(idx) - 1;
        let m = if n >= 0 { t - n } else { p - n - 1 };
        self.stack.reverse(p as usize, m as usize);
        self.stack.reverse(m as usize + 1, t as usize);
        self.stack.reverse(p as usize, t as usize);
    }
    fn set_top(&mut self, idx: isize) {
        let new_top = self.stack.abs_index(idx);
        if new_top < 0 {
            panic!("Stack underflow!");
        }
        let n = self.stack.top() - new_top;
        if n > 0 {
            for _ in 0..n {
                self.stack.pop();
            }
        } else {
            for _ in n..0 {
                self.stack.push(LuaValue::Nil);
            }
        }
    }

    /* =========================== Access Methods =========================== */
    fn type_name(&self, t: LuaType) -> &str {
        match t {
            LUA_TNONE => "no value",
            LUA_TNIL => "nil",
            LUA_TBOOLEAN => "boolean",
            LUA_TNUMBER => "number",
            LUA_TSTRING => "string",
            LUA_TTABLE => "table",
            LUA_TFUNCTION => "function",
            LUA_TTHREAD => "thread",
            _ => "userdata",
        }
    }
    fn type_id(&self, idx: isize) -> LuaType {
        if self.stack.is_valid(idx) { self.stack.get(idx).type_id() } else { LUA_TNONE }
    }
    fn is_none(&self, idx: isize) -> bool {
        self.type_id(idx) == LUA_TNONE
    }
    fn is_nil(&self, idx: isize) -> bool {
        self.type_id(idx) == LUA_TNIL
    }

    fn is_none_or_nil(&self, idx: isize) -> bool {
        self.type_id(idx) <= LUA_TNIL
    }
    fn is_boolean(&self, idx: isize) -> bool {
        self.type_id(idx) == LUA_TBOOLEAN
    }

    fn is_string(&self, idx: isize) -> bool {
        let t = self.type_id(idx);
        t == LUA_TSTRING || t == LUA_TNUMBER
    }

    fn is_number(&self, idx: isize) -> bool {
        self.to_numberx(idx).is_some()
    }

    fn is_integer(&self, idx: isize) -> bool {
        match self.stack.get(idx) {
            LuaValue::Integer(_) => true,
            _ => false,
        }
    }


    fn is_table(&self, idx: isize) -> bool {
        self.type_id(idx) == LUA_TTABLE
    }

    fn is_thread(&self, idx: isize) -> bool {
        self.type_id(idx) == LUA_TTHREAD
    }

    fn is_function(&self, idx: isize) -> bool {
        self.type_id(idx) == LUA_TFUNCTION
    }

    fn to_boolean(&self, idx: isize) -> bool {
        self.stack.get(idx).to_boolean()
    }

    fn to_integer(&self, idx: isize) -> i64 {
        self.to_integerx(idx).unwrap_or(0)
    }

    fn to_integerx(&self, idx: isize) -> std::option::Option<i64> {
        match self.stack.get(idx) {
            LuaValue::Integer(i) => Some(i),
            _ => None,
        }
    }

    fn to_number(&self, idx: isize) -> f64 {
        self.to_numberx(idx).unwrap_or(0f64)
    }

    fn to_numberx(&self, idx: isize) -> std::option::Option<f64> {
        match self.stack.get(idx) {
            LuaValue::Number(n) => Some(n),
            LuaValue::Number(i) => Some(i as f64),
            _ => None,
        }
    }
    fn to_string(&self, idx: isize) -> std::string::String {
        self.to_stringx(idx).unwrap_or("".to_string())
    }
    fn to_stringx(&self, idx: isize) -> std::option::Option<std::string::String> {
        match self.stack.get(idx) {
            LuaValue::Str(s) => Some(s),
            LuaValue::Integer(i) => Some(i.to_string()),
            LuaValue::Number(n) => Some(n.to_string()),
            _ => None,
        }
    }

    /* =========================== Push Methods =========================== */
    fn push_nil(&mut self) {
        self.stack.push(LuaValue::Nil);
    }
    fn push_boolean(&mut self, b: bool) {
        self.stack.push(LuaValue::Boolean(b));
    }
    fn push_integer(&mut self, i: i64) {
        self.stack.push(LuaValue::Integer(i));
    }
    fn push_number(&mut self, n: f64) {
        self.stack.push(LuaValue::Number(n));
    }
    fn push_string(&mut self, s: std::string::String) {
        self.stack.push(LuaValue::Str(s));
    }

}