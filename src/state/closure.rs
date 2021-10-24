use crate::binary::chunk::Prototype;
use crate::number::math;
use std::hash::Hash;
use std::rc::Rc;
use std::cell::RefCell;
use crate::api::RustFn;
use super::lua_value::LuaValue;

#[derive(Clone)]
pub struct UpValue {
    pub val: Rc<RefCell<LuaValue>>,
}

pub struct Closure {
    pub proto: Rc<Prototype>,   // lua closure
    pub rust_fn: Option<RustFn>,// rust closure
    pub upvals: Vec<Option<UpValue>>,
    rdm: usize,
}

impl Hash for Closure {
    fn hash<H>(&self, state: &mut H) where H: std::hash::Hasher {
        self.rdm.hash(state);
    }
}

impl Closure {
    pub fn new_dummy_closure() -> Closure {
        Closure {
            proto: new_dummy_prototype(),
            rust_fn: None,
            upvals: Vec::new(),
            rdm: math::random(),
        }
    }

    pub fn new_lua_closure(proto: Rc<Prototype>) -> Closure {
        let n_upvals = proto.upvalues.len();
        Closure {
            proto,
            rust_fn: None,
            upvals: vec![None; n_upvals],
            rdm: math::random(),
        }
    }

    pub fn new_rust_closure(f: RustFn, n_upvals: usize) -> Closure {
        Closure {
            proto: new_dummy_prototype(),
            rust_fn: Some(f),
            upvals: vec![None; n_upvals],
            rdm: math::random(),
        }
    }
}

fn new_dummy_prototype() -> Rc<Prototype> {
    Rc::new(Prototype {
        source: None, // debug
        line_defined: 0,
        last_line_defined: 0,
        num_params: 0,
        is_vararg: 0,
        max_stack_size: 0,
        code: vec![],
        constants: vec![],
        upvalues: vec![],
        protos: vec![],
        line_info: vec![],     // debug
        loc_vars: vec![],      // debug
        upvalue_names: vec![], // debug
    })
}
