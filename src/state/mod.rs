mod api_arith;
mod api_compare;
mod api_stack;
mod closure;
mod lua_stack;
mod lua_state;
mod lua_table;
mod lua_value;

pub use self::lua_state::LuaState;
use crate::binary::chunk::Prototype;
use std::rc::Rc;
use std::cell::RefCell;

pub fn new_lua_state(stack_size: usize, proto: Rc<Prototype>) -> Rc<RefCell<LuaState>> {
    let ls = Rc::new(RefCell::new(LuaState::new()));
    ls.borrow_mut().push_frame(self::lua_stack::LuaStack::new(
            stack_size,
            Rc::new(RefCell::new(self::closure::Closure::new_lua_closure(proto)))
    ));
    ls.borrow_mut().stack_mut().state = Some(Rc::downgrade(&ls));
    ls
}
