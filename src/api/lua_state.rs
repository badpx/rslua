use super::consts::{ArithOp, CompareOp, LuaType};

pub trait LuaState {
    /* basic stack manipulation */
    fn get_top(&self) -> isize;
    fn abs_index(&self, idx: isize) -> isize;
    fn check_stack(&mut self, n: usize) -> bool;
    fn pop(&mut self, n: usize);
    fn copy(&mut self, from_idx: isize, to_idx: isize);
    fn push_value(&mut self, idx: isize);
    fn replace(&mut self, idx: isize);
    fn insert(&mut self, idx: isize);
    fn remove(&mut self, idx: isize);
    fn rotate(&mut self, idx: isize, n: isize);
    fn set_top(&mut self, idx: isize);
    /* access methods (stack -> rust) */
    fn type_name(&self, tp: LuaType) -> &str; // TODO
    fn type_id(&self, idx: isize) -> LuaType; // `type` is a keyword
    fn is_none(&self, idx: isize) -> bool;
    fn is_nil(&self, idx: isize) -> bool;
    fn is_none_or_nil(&self, idx: isize) -> bool;
    fn is_boolean(&self, idx: isize) -> bool;
    fn is_integer(&self, idx: isize) -> bool;
    fn is_number(&self, idx: isize) -> bool;
    fn is_string(&self, idx: isize) -> bool;
    fn is_table(&self, idx: isize) -> bool;
    fn is_thread(&self, idx: isize) -> bool;
    fn is_function(&self, idx: isize) -> bool;
    fn to_boolean(&self, idx: isize) -> bool;
    fn to_integer(&self, idx: isize) -> i64;
    fn to_integerx(&self, idx: isize) -> Option<i64>;
    fn to_number(&self, idx: isize) -> f64;
    fn to_numberx(&self, idx: isize) -> Option<f64>;
    fn to_string(&self, idx: isize) -> String;
    fn to_stringx(&self, idx: isize) -> Option<String>;
    /* push methods (rust -> stack) */
    fn push_nil(&mut self);
    fn push_boolean(&mut self, b: bool);
    fn push_integer(&mut self, n: i64);
    fn push_number(&mut self, n: f64);
    fn push_string(&mut self, s: String);
    /* comparison and arithmetic methods */
    fn arith(&mut self, op: ArithOp);
    fn compare(&self, idx1: isize, idx2: isize, op: CompareOp) -> bool;
    /* miscellaneous methods */
    fn len(&mut self, idx: isize);
    fn concat(&mut self, n: isize);
    /* get functions (Lua -> stack) */
    fn new_table(&mut self);
    fn create_table(&mut self, narr: usize, nrec: usize);
    fn get_table(&mut self, idx: isize) -> i8;
    fn get_field(&mut self, idx: isize, k: &str) -> i8;
    fn get_i(&mut self, idx: isize, i: i64) -> i8;
    // set functions (stack -> Lua)
    fn set_table(&mut self, idx: isize);
    fn set_field(&mut self, idx: isize, k: &str);
    fn set_i(&mut self, idx: isize, i: i64);
    // call functions
    fn load(&mut self, chunk: Vec<u8>, chunk_name: &str, mode: &str) -> u8;
    fn call(&mut self, nargs: usize, nresults: isize);
    // call native functions
    fn push_rust_fn(&mut self, f: RustFn);
    fn is_rust_fn(&self, idx: isize) -> bool;
    fn to_rust_fn(&mut self, idx: isize) -> Option<RustFn>;
    // access global table
    fn push_global_table(&mut self);
    fn get_global(&mut self, name: &str) -> LuaType;
    fn set_global(&mut self, name: &str);
    fn register(&mut self, name: &str, f: RustFn);
}

pub type RustFn = fn(&dyn LuaState) -> usize;
