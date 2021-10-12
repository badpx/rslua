use crate::api::lua_vm::LuaVM;
use super::instruction::Instruction;

// R(A) = R(B). P.S. move is keyword of Rust
pub fn _move(i: u32, vm: &mut dyn LuaVM) {
    let (a, b, _) = i.abc();
    vm.copy(a + 1, b + 1);
}

pub fn jmp(i: u32, vm: &mut dyn LuaVM) {
    let (a, sbx) = i.a_sbx();
    vm.add_pc(sbx);

    if a != 0 {
        todo!();
    }
}