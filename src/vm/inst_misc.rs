use crate::api::LuaVM;
use super::instruction::Instruction;

/* 
                MOVE Instruction
                   R(A) = R(B)
    +---------+---------+---------+---------+
    |   B:1   |    C:   |   A:3   |   MOVE  |
    +---------+---------+---------+---------+

        +---------+           +---------+
        |    d    |     +---->|    b    | <-A 
        +---------+     |     +---------+
        |    c    |     |     |    c    |
        +---------+     |     +---------+
        |    b    |-----+     |    b    |
        +---------+           +---------+
    B-> |    a    |           |    a    |
        +---------+           +---------+
         registers             registers
*/
pub fn move_(i: u32, vm: &mut dyn LuaVM) {
    let (a, b, _) = i.abc();
    vm.copy(b + 1, a + 1);
}

pub fn jmp(i: u32, vm: &mut dyn LuaVM) {
    let (a, sbx) = i.a_sbx();
    vm.add_pc(sbx);

    if a != 0 {
        todo!();
    }
}