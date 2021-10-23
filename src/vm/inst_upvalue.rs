
use crate::api::LuaVM;
use super::instruction::Instruction;

pub fn get_tab_up(i: u32, vm: &mut dyn LuaVM) {
    let (a, _, c) = i.abc();
    vm.push_global_table();
    vm.get_rk(c);
    vm.get_table(-2);
    vm.replace(a + 1);
    vm.pop(1);
}
