use super::api_arith;
use super::api_compare::compare;
use super::closure::UpValue;
use super::closure::Closure;
use super::lua_stack::LuaStack;
use super::lua_state::LuaState;
use super::lua_value::LuaValue;
use crate::api::consts::*;
use crate::api::{LuaAPI, LuaVM, RustFn};
use crate::vm::instruction::Instruction;
use std::rc::Rc;
use std::cell::RefCell;

const LUAVAL_RIDX_GLOBALS: LuaValue = LuaValue::Integer(LUA_RIDX_GLOBALS);

impl LuaAPI for LuaState {
    /* =========================== Basic Methods =========================== */
    fn get_top(&self) -> isize {
        self.stack().top()
    }

    fn abs_index(&self, idx: isize) -> isize {
        self.stack().abs_index(idx)
    }

    fn check_stack(&mut self, n: usize) -> bool {
        self.stack_mut().check(n);
        return true; // Never fails
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
        let val = self.stack().get(from);
        self.stack_mut().set(to, val);
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
        let val = self.stack().get(idx);
        self.stack_mut().push(val);
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
        let val = self.stack_mut().pop();
        self.stack_mut().set(idx, val);
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
        let t = self.stack().top() - 1;
        let p = self.stack().abs_index(idx) - 1;
        let m = if n >= 0 { t - n } else { p - n - 1 };
        self.stack_mut().reverse(p as usize, m as usize);
        self.stack_mut().reverse(m as usize + 1, t as usize);
        self.stack_mut().reverse(p as usize, t as usize);
    }

    fn set_top(&mut self, idx: isize) {
        self.stack_mut().set_top(idx);
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
        if self.stack().is_valid(idx) {
            self.stack().get(idx).type_id()
        } else {
            LUA_TNONE
        }
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
        match self.stack().get(idx) {
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
        self.stack().get(idx).to_boolean()
    }

    fn to_integer(&self, idx: isize) -> i64 {
        self.to_integerx(idx).unwrap_or(0)
    }

    fn to_integerx(&self, idx: isize) -> std::option::Option<i64> {
        self.stack().get(idx).to_integer()
    }

    fn to_number(&self, idx: isize) -> f64 {
        self.to_numberx(idx).unwrap_or(0f64)
    }

    fn to_numberx(&self, idx: isize) -> std::option::Option<f64> {
        self.stack().get(idx).to_number()
    }

    fn to_string(&self, idx: isize) -> std::string::String {
        self.to_stringx(idx).unwrap_or("".to_string())
    }

    fn to_stringx(&self, idx: isize) -> std::option::Option<std::string::String> {
        match self.stack().get(idx) {
            LuaValue::Str(s) => Some(s),
            LuaValue::Integer(i) => Some(i.to_string()),
            LuaValue::Number(n) => Some(n.to_string()),
            _ => None,
        }
    }

    /* =========================== Push Methods =========================== */
    fn push_nil(&mut self) {
        self.stack_mut().push(LuaValue::Nil);
    }
    fn push_boolean(&mut self, b: bool) {
        self.stack_mut().push(LuaValue::Boolean(b));
    }
    fn push_integer(&mut self, i: i64) {
        self.stack_mut().push(LuaValue::Integer(i));
    }
    fn push_number(&mut self, n: f64) {
        self.stack_mut().push(LuaValue::Number(n));
    }
    fn push_string(&mut self, s: std::string::String) {
        self.stack_mut().push(LuaValue::Str(s));
    }

    /* ================= comparison and arithmetic methods ================= */
    /*            Arith
        (Binary opration, such as +)
        +-------+       +-------+
        |       |       |       |
        +-------+       +-------+
        |   c   |---+   |       |
        +-------+   |   +-------+
        |   b   |---+-->| b + c |
        +-------+       +-------+
        |   a   |       |   a   |
        +-------+       +-------+

        (Unary opration, such as ~)
        +-------+       +-------+
        |       |       |       |
        +-------+       +-------+
        |   c   |------>|   ~c  |
        +-------+       +-------+
        |   b   |       |   b   |
        +-------+       +-------+
        |   a   |       |   a   |
        +-------+       +-------+
    */
    fn arith(&mut self, op: ArithOp) {
        if op != LUA_OPUNM && op != LUA_OPBNOT {
            let b = self.stack_mut().pop();
            let a = self.stack_mut().pop();
            if let Some(ret) = api_arith::_arith(&a, &b, op) {
                self.stack_mut().push(ret);
                return;
            }
        } else {
            let a = self.stack_mut().pop();
            if let Some(ret) = api_arith::_arith(&a, &a, op) {
                self.stack_mut().push(ret);
                return;
            }
        }
        panic!("Arithmetic error!");
    }

    fn compare(&self, idx1: isize, idx2: isize, op: CompareOp) -> bool {
        if !self.stack().is_valid(idx1) || !self.stack().is_valid(idx2) {
            false
        } else {
            let a = self.stack().get(idx1);
            let b = self.stack().get(idx2);
            match op {
                LUA_OPEQ | LUA_OPLT | LUA_OPLE => compare(&a, &b, op),
                _ => panic!("Invalid compare operation"),
            }
        }
    }

    /* miscellaneous methods */
    /*
                  len(2)
        +-------+       +-------+
        |       |   +-->|   #b  |
        +-------+   |   +-------+
        |   c   |   |   |   c   |
        +-------+   |   +-------+
        |   b   |---+   |   b   |
        +-------+       +-------+
        |   a   |       |   a   |
        +-------+       +-------+
    */
    fn len(&mut self, idx: isize) {
        let val = self.stack().get(idx);
        if let LuaValue::Str(s) = val {
            self.stack_mut().push(LuaValue::Integer(s.len() as i64));
        } else if let LuaValue::Table(tbl) = val {
            self.stack_mut().push(LuaValue::Integer(tbl.borrow().len() as i64));
        } else {
            panic!("TODO: need support more type!");
        }
    }

    /*
                concat(2)
        +-------+       +-------+
        |       |       |       |
        +-------+       +-------+
        |   c   |---+   |       |
        +-------+   |   +-------+
        |   b   |---+-->|  b..c |
        +-------+       +-------+
        |   a   |       |   a   |
        +-------+       +-------+
    */
    fn concat(&mut self, n: isize) {
        if n == 0 {
            self.stack_mut().push(LuaValue::Str("".to_string()));
        } else if n > 1 {
            for _ in 1..n {
                if self.is_string(-1) && self.is_string(-2) {
                    let s2 = self.to_string(-1);
                    let s1 = self.to_string(-2);
                    self.stack_mut().pop();
                    self.stack_mut().pop();
                    self.stack_mut().push(LuaValue::Str(s1 + &s2));
                } else {
                    panic!("Concatenation error!");
                }
            }
        }
        // n == 1, do nothing.
    }

    /* get functions (Lua -> stack) */
    fn create_table(&mut self, narr: usize, nrec: usize) {
        self.stack_mut().push(LuaValue::new_table(narr, nrec));
    }

    fn new_table(&mut self) {
        self.create_table(0, 0);
    }

    /*
               get_table(2)
        +-------+       +-------+
        |   k   |-{[]}->|  t[k] |
        +-------+   |   +-------+
        |   c   |   |   |   c   |
        +-------+   |   +-------+
        |   t   |---+   |   b   |
        +-------+       +-------+
        |   a   |       |   a   |
        +-------+       +-------+
    */
    fn get_table(&mut self, idx: isize) -> i8 {
        let t = self.stack().get(idx);
        let k = self.stack_mut().pop();
        self._get_table(&t, &k)
    }

    /*
            get_field(2, "k")
        +-------+  "k"    +-------+
        |       |   |  +->|  t.k  |
        +-------+   |  |  +-------+
        |   c   |  {[]}+  |   c   |
        +-------+   |     +-------+
        |   t   |---+     |   b   |
        +-------+         +-------+
        |   a   |         |   a   |
        +-------+         +-------+
    */
    fn get_field(&mut self, idx: isize, k: &str) -> LuaType {
        let t = self.stack().get(idx);
        let k = LuaValue::Str(k.to_string());
        self._get_table(&t, &k)
    }

    /*
                get_i(2,3)
        +-------+   3     +-------+
        |       |   |  +->|  t[3] |
        +-------+   |  |  +-------+
        |   c   |  {[]}+  |   c   |
        +-------+   |     +-------+
        |   t   |---+     |   b   |
        +-------+         +-------+
        |   a   |         |   a   |
        +-------+         +-------+
    */
    fn get_i(&mut self, idx: isize, i: i64) -> i8 {
        let t = self.stack().get(idx);
        let k = LuaValue::Integer(i);
        self._get_table(&t, &k)
    }

    // set functions (stack -> Lua)

    /*          set_table(2)
                  t[k]=v
        +-------+ | |  |  +-------+
        |   v   |-+-+--+  |       |
        +-------+ | |     +-------+
        |   k   |-+-+     |       |
        +-------+ |       +-------+
        |   t   |-+       |   t   |
        +-------+         +-------+
        |   a   |         |   a   |
        +-------+         +-------+
    */
    fn set_table(&mut self, idx: isize) {
        let t = self.stack().get(idx);
        let v = self.stack_mut().pop();
        let k = self.stack_mut().pop();
        LuaState::_set_table(&t, k, v);
    }

    /*      set_field(2,"k")
                  t.k=v
        +-------+ |   |  +-------+
        |   v   |-+---+  |       |
        +-------+ |      +-------+
        |   c   | +      |   c   |
        +-------+ |      +-------+
        |   t   |-+      |   t   |
        +-------+        +-------+
        |   a   |        |   a   |
        +-------+        +-------+
    */
    fn set_field(&mut self, idx: isize, k: &str) {
        let t = self.stack().get(idx);
        let k = LuaValue::Str(k.to_string());
        let v = self.stack_mut().pop();
        LuaState::_set_table(&t, k, v);
    }

    /*        set_field(2,3)
                  t[3]=v
        +-------+ |    | +-------+
        |   v   |-+----+ |       |
        +-------+ |      +-------+
        |   c   | +      |   c   |
        +-------+ |      +-------+
        |   t   |-+      |   t   |
        +-------+        +-------+
        |   a   |        |   a   |
        +-------+        +-------+
    */
    fn set_i(&mut self, idx: isize, i: i64) {
        let t = self.stack().get(idx);
        let v = self.stack_mut().pop();
        let k = LuaValue::Integer(i);
        LuaState::_set_table(&t, k, v);
    }

    // Load chunk to top of stack
    fn load(&mut self, chunk: Vec<u8>, /*chunk_name*/ _: &str, /*mode*/ _: &str) -> u8 {
        let proto = crate::binary::undump(chunk);
        let upvalues = proto.upvalues.len();
        let c = LuaValue::new_lua_closure(proto);
        if upvalues > 0 {
            if let LuaValue::Table(tbl) = &self.registry {
                let env = tbl.borrow().get(&LUAVAL_RIDX_GLOBALS);
                if let LuaValue::Function(ref closure) = c {
                    closure.borrow_mut().upvals[0] = Some(UpValue{ val: Rc::new(RefCell::new(env)) });
                }
            } else {
                panic!("No registry table!");
            }
        }
        self.stack_mut().push(c);
        0 // TODO:
    }

    fn call(&mut self, nargs: usize, nresults: isize) {
        let val = self.stack().get(-(nargs as isize + 1));
        if let LuaValue::Function(c) = val {
            // DEBUG info
            //println!(" Call {}<{}, {}>", c.proto.source.clone().unwrap(), c.proto.line_defined, c.proto.last_line_defined);

            if c.borrow().rust_fn.is_none() {
                self.call_lua_closure(nargs, nresults, c);
            } else {
                self.call_rust_closure(nargs, nresults, c);
            }
        } else {
            panic!("Not function!");
        }
    }

    fn push_rust_fn(&mut self, f: RustFn) {
        self.stack_mut()
            .push(LuaValue::Function(Rc::new(RefCell::new(Closure::new_rust_closure(f, 0)))));
    }

    fn is_rust_fn(&self, idx: isize) -> bool {
        match self.stack().get(idx) {
            LuaValue::Function(c) => c.borrow().rust_fn.is_some(),
            _ => false,
        }
    }

    fn to_rust_fn(&mut self, idx: isize) -> Option<RustFn> {
        match self.stack().get(idx) {
            LuaValue::Function(c) => c.borrow().rust_fn,
            _ => None,
        }
    }

    fn push_global_table(&mut self) {
        if let LuaValue::Table(t) = &self.registry {
            let global = t.borrow().get(&LUAVAL_RIDX_GLOBALS);
            self.stack_mut().push(global);
        }
    }

    /*        get_global("k")
                       
        +-------+        +-------+
        |       |     -->| _G.k  |
        +-------+        +-------+
        |   c   |        |   c   |
        +-------+        +-------+
        |   b   |        |   b   |
        +-------+        +-------+
        |   a   |        |   a   |
        +-------+        +-------+
    */
    fn get_global(&mut self, name: &str) -> LuaType {
        if let LuaValue::Table(t) = &self.registry {
            let global = t.borrow().get(&LUAVAL_RIDX_GLOBALS);
            let k = LuaValue::Str(name.to_string());
            self._get_table(&global, &k)
        } else {
            LUA_TNONE
        }
    }

    /*        set_global("k")
                       
        +-------+        +-------+
        |   v   |----+   |       |
        +-------+    |   +-------+
        |   c   |    V   |   c   |
        +-------+ _G.k=v +-------+
        |   b   |        |   b   |
        +-------+        +-------+
        |   a   |        |   a   |
        +-------+        +-------+
    */
    fn set_global(&mut self, name: &str) {
        if let LuaValue::Table(t) = &self.registry {
            let global = t.borrow().get(&LUAVAL_RIDX_GLOBALS);
            let v = self.stack_mut().pop();
            let k = LuaValue::Str(name.to_string());
            LuaState::_set_table(&global, k, v);
        }
    }

    fn register(&mut self, name: &str, f: RustFn) {
        self.push_rust_fn(f);
        self.set_global(name);
    }

    /*     push_rust_closure(f, 2)
                       
        +-------+          +-------+
        |   u2  |---+      |       |
        +-------+   |      +-------+
        |   u1  |-{close}->|   f   |
        +-------+          +-------+
        |   b   |          |   b   |
        +-------+          +-------+
        |   a   |          |   a   |
        +-------+          +-------+
    */
    fn push_rust_closure(&mut self, f: RustFn, n: usize) {
        let mut closure = Closure::new_rust_closure(f, n);
        for _ in 0..n {
            let val = self.stack_mut().pop();
            closure.upvals[n - 1] = Some(UpValue{val: Rc::new(RefCell::new(val))});
        }
        self.stack_mut().push(LuaValue::Function(Rc::new(RefCell::new(closure))));
    }
}

impl LuaState {
    fn _get_table(&mut self, t: &LuaValue, k: &LuaValue) -> i8 {
        if let LuaValue::Table(tbl) = t {
            let v = tbl.borrow().get(k);
            let type_id = v.type_id();
            self.stack_mut().push(v);
            type_id
        } else {
            todo!()
        }
    }

    fn _set_table(t: &LuaValue, k: LuaValue, v: LuaValue) {
        if let LuaValue::Table(tbl) = t {
            tbl.borrow_mut().put(k, v);
        } else {
            todo!();
        }
    }

    fn call_lua_closure(&mut self, nargs: usize, nresults: isize, c: Rc<RefCell<Closure>>) {
        let nregs = c.borrow().proto.max_stack_size as usize;
        let nparams = c.borrow().proto.num_params as usize;
        let is_vararg = c.borrow().proto.is_vararg == 1;

        if let Some(state) = &self.stack().state {
            // create new lua stack
            let mut new_stack = LuaStack::new(nregs + LUA_MINSTACK, c);
            new_stack.state = Some(state.clone());

            // pop args and func
            let mut args = self.stack_mut().pop_n(nargs);
            self.stack_mut().pop(); // pop func
            if nargs > nparams {
                // check if varargs func
                for _ in nparams..nargs {
                    new_stack.varargs.push(args.pop().unwrap());
                }
                if is_vararg {
                    new_stack.varargs.reverse();
                } else {
                    new_stack.varargs.clear();
                }
            }
            new_stack.push_n(args, nparams as isize);
            new_stack.set_top(nregs as isize);

            // run closure
            self.push_frame(new_stack);
            self.run_lua_closure();
            new_stack = self.pop_frame();

            // return results
            if nresults != 0 {
                let nrets = new_stack.top() as usize - nregs;
                let results = new_stack.pop_n(nrets);
                self.check_stack(nrets);
                self.stack_mut().push_n(results, nresults);
            }
        } else {
            panic!("Frame stack is empty!");
        }
    }

    fn run_lua_closure(&mut self) {
        loop {
            let inst = self.fetch();
            inst.execute(self);

            // DEBUG info
            /*
            let line = format!("{}", self.stack().closure.proto.line_info[self.pc() as usize - 1]);
            print!("{:04}\t[{}]\t{} ", self.pc(), line, inst.opname());
            print_oprands(inst);
            println!("\t{:?}", self.stack()._raw_data());
            */

            if inst.opcode() == crate::vm::opcodes::OP_RETURN {
                break;
            }
        }
    }

    fn call_rust_closure(&mut self, nargs: usize, nresults: isize, c: Rc<RefCell<Closure>>) {
        let rust_fn = c.borrow().rust_fn.unwrap();
        if let Some(state) = &self.stack().state {
            // create new lua stack
            let mut new_stack = LuaStack::new(nargs + LUA_MINSTACK, c);
            new_stack.state = Some(state.clone());

            if nargs > 0 {
                let args = self.stack_mut().pop_n(nargs);
                new_stack.push_n(args, nargs as isize);
            }
            self.stack_mut().pop();

            // run closure
            self.push_frame(new_stack);
            let r = rust_fn(self);
            new_stack = self.pop_frame();

            if nresults != 0 {
                let results = new_stack.pop_n(r);
                self.check_stack(results.len());
                self.stack_mut().push_n(results, nresults);
            }
        } else {
            panic!("Frame stack is empty!");
        }
    }

    fn lua_upvalue_index(i: isize) -> isize {
        LUA_REGISTRY_INDEX - i
    }
}

use crate::vm::opcodes::*;
fn print_oprands(i: u32) {
    match i.opmode() {
        OP_MODE_ABC => {
            let (a, b, c) = i.abc();
            print!("{}", a);
            if i.b_mode() != OP_ARG_N {
                if b > 0xFF {
                    print!(" {}", -1 - (b & 0xFF));
                } else {
                    print!(" {}", b);
                }
            }
            if i.c_mode() != OP_ARG_N {
                if c > 0xFF {
                    print!(" {}", -1 - (c & 0xFF));
                } else {
                    print!(" {}", c);
                }
            }
        }
        OP_MODE_ABX => {
            let (a, bx) = i.a_bx();
            print!(" {}", a);
            if i.b_mode() == OP_ARG_K {
                print!(" {}", -1 - bx);
            } else if i.b_mode() == OP_ARG_U {
                print!(" {}", bx);
            }
        }
        OP_MODE_ASBX => {
            let (a, sbx) = i.a_sbx();
            print!("{} {}", a, sbx);
        }
        OP_MODE_AX => {
            let ax = i.ax();
            print!("{}", -1 - ax);
        }
        _ => unreachable!(),
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::reader::tests::LUA_FOR_LOOP;
    use crate::state::new_lua_state;

    #[test]
    fn stack() {
        let proto = crate::binary::undump(LUA_FOR_LOOP.to_vec());
        let ls = new_lua_state(proto.max_stack_size as usize, proto);
        assert_eq!(*ls.borrow().stack()._raw_data(), Vec::<LuaValue>::new());
        ls.borrow_mut().push_boolean(true);
        assert_eq!(*ls.borrow().stack()._raw_data(), vec![LuaValue::Boolean(true)]);
        ls.borrow_mut().push_integer(10);
        assert_eq!(
            *ls.borrow().stack()._raw_data(),
            vec![LuaValue::Boolean(true), LuaValue::Integer(10)]
        );
        ls.borrow_mut().push_nil();
        assert_eq!(
            *ls.borrow().stack()._raw_data(),
            vec![LuaValue::Boolean(true), LuaValue::Integer(10), LuaValue::Nil]
        );
        ls.borrow_mut().push_string("hello".to_string());
        assert_eq!(
            *ls.borrow().stack()._raw_data(),
            vec![
                LuaValue::Boolean(true),
                LuaValue::Integer(10),
                LuaValue::Nil,
                LuaValue::Str("hello".to_string())
            ]
        );
        ls.borrow_mut().push_value(-4);
        assert_eq!(
            *ls.borrow().stack()._raw_data(),
            vec![
                LuaValue::Boolean(true),
                LuaValue::Integer(10),
                LuaValue::Nil,
                LuaValue::Str("hello".to_string()),
                LuaValue::Boolean(true)
            ]
        );
        ls.borrow_mut().replace(3);
        assert_eq!(
            *ls.borrow().stack()._raw_data(),
            vec![
                LuaValue::Boolean(true),
                LuaValue::Integer(10),
                LuaValue::Boolean(true),
                LuaValue::Str("hello".to_string())
            ]
        );
        ls.borrow_mut().set_top(6);
        assert_eq!(
            *ls.borrow().stack()._raw_data(),
            vec![
                LuaValue::Boolean(true),
                LuaValue::Integer(10),
                LuaValue::Boolean(true),
                LuaValue::Str("hello".to_string()),
                LuaValue::Nil,
                LuaValue::Nil
            ]
        );
        ls.borrow_mut().remove(-3);
        assert_eq!(
            *ls.borrow().stack()._raw_data(),
            vec![
                LuaValue::Boolean(true),
                LuaValue::Integer(10),
                LuaValue::Boolean(true),
                LuaValue::Nil,
                LuaValue::Nil
            ]
        );
        ls.borrow_mut().set_top(-5);
        assert_eq!(*ls.borrow().stack()._raw_data(), vec![LuaValue::Boolean(true)]);
    }

    #[test]
    fn arith() {
        let proto = crate::binary::undump(LUA_FOR_LOOP.to_vec());
        let ls = new_lua_state(proto.max_stack_size as usize, proto);
        ls.borrow_mut().push_integer(1);
        assert_eq!(*ls.borrow().stack()._raw_data(), vec![LuaValue::Integer(1)]);
        ls.borrow_mut().push_string("2.0".to_string());
        assert_eq!(
            *ls.borrow().stack()._raw_data(),
            vec![LuaValue::Integer(1), LuaValue::Str("2.0".to_string())]
        );
        ls.borrow_mut().push_string("3.0".to_string());
        assert_eq!(
            *ls.borrow().stack()._raw_data(),
            vec![
                LuaValue::Integer(1),
                LuaValue::Str("2.0".to_string()),
                LuaValue::Str("3.0".to_string())
            ]
        );
        ls.borrow_mut().push_number(4.0);
        assert_eq!(
            *ls.borrow().stack()._raw_data(),
            vec![
                LuaValue::Integer(1),
                LuaValue::Str("2.0".to_string()),
                LuaValue::Str("3.0".to_string()),
                LuaValue::Number(4.0)
            ]
        );

        ls.borrow_mut().arith(LUA_OPADD);
        assert_eq!(
            *ls.borrow().stack()._raw_data(),
            vec![
                LuaValue::Integer(1),
                LuaValue::Str("2.0".to_string()),
                LuaValue::Number(7.0)
            ]
        );
        ls.borrow_mut().arith(LUA_OPBNOT);
        assert_eq!(
            *ls.borrow().stack()._raw_data(),
            vec![
                LuaValue::Integer(1),
                LuaValue::Str("2.0".to_string()),
                LuaValue::Integer(-8)
            ]
        );
        ls.borrow_mut().len(2);
        assert_eq!(
            *ls.borrow().stack()._raw_data(),
            vec![
                LuaValue::Integer(1),
                LuaValue::Str("2.0".to_string()),
                LuaValue::Integer(-8),
                LuaValue::Integer(3)
            ]
        );
        ls.borrow_mut().concat(3);
        assert_eq!(
            *ls.borrow().stack()._raw_data(),
            vec![LuaValue::Integer(1), LuaValue::Str("2.0-83".to_string())]
        );
    }
}
