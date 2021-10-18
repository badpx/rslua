use crate::binary::chunk::Prototype;
use crate::number::math;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

pub struct Closure {
    pub proto: Rc<Prototype>,
    rdm: usize,
}

impl Hash for Closure {
    fn hash<H>(&self, state: &mut H) where H: std::hash::Hasher {
        self.rdm.hash(state);
    }
}

impl Closure {
    pub fn new(proto: Rc<Prototype>) -> Closure {
        Closure {
            proto,
            rdm: math::random(),
        }
    }
}