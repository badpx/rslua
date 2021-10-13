
use crate::api::LuaVM;
use super::instruction::Instruction;

/* 
                LOADNIL Instruction
        R(A), R(A+1), ..., R(A+B) := nil
    +---------+---------+---------+---------+
    |   B:3   |    C:   |   A:0   | LOADNIL |
    +---------+---------+---------+---------+

        +---------+           +---------+
        |    d    |     +---->|   nil   | <-A+B
        +---------+     |     +---------+
        |    c    |     |     |   nil   | <-A+2
        +---------+     |     +---------+
        |    b    |-----+     |   nil   | <-A+1
        +---------+           +---------+
        |    a    |           |   nil   | <-A
        +---------+           +---------+
         registers             registers
*/
pub fn load_nil(i: u32, vm: &mut dyn LuaVM) {
    let (a, b, _) = i.abc();
    let a = a + 1;
    vm.push_nil();
    for i in a..=a+b {
        vm.copy(-1, i);
    }
    vm.pop(1);
}

/* 
*/
/* 
                LOADBOOL Instruction
        R(A) := (bool)B; if (C) then pc++

                    +--{if true}--> JMP 1
    +---------+-----|---+---------+---------+
    |   B: 0  |   C: 1  |   A:2   | LOADBOOL|
    +-----|---+---------+---------+---------+
          +-------------+
        +---------+     |     +---------+
        |    d    |     |     |    d    |
        +---------+     |     +---------+
        |    c    |     +---->|  false  | <-A
        +---------+           +---------+
        |    b    |           |    b    |
        +---------+           +---------+
        |    a    |           |    a    |
        +---------+           +---------+
         registers             registers
*/
pub fn load_bool(i: u32, vm: &mut dyn LuaVM) {
    let (a, b, c) = i.abc();
    vm.push_boolean(b != 0);
    vm.replace(a + 1);
    if c != 0 {
        vm.add_pc(1);
    }
}

/* 
                LOADK Instruction
                R(A) := Kst(Bx)
    +-------------------+---------+---------+
    |       Bx: 2       |   A:3   |  LOADK  |
    +-------------------+---------+---------+

                              +---------+
                        +---->|  "foo"  | <-A
        +---------+     |     +---------+
   Bx-> |  "foo"  |-----+     |    c    |
        +---------+           +---------+
        |    2    |           |    b    |
        +---------+           +---------+
        |    1    |           |    a    |
        +---------+           +---------+
         constants             registers
*/
pub fn load_k(i: u32, vm: &mut dyn LuaVM) {
    let (a, bx) = i.a_bx();
    vm.get_const(bx);
    vm.replace(a + 1);
}

pub fn load_kx(i: u32, vm: &mut dyn LuaVM) {
    let (a, _) = i.a_bx();
    let ax = vm.fetch().ax();
    vm.get_const(ax);
    vm.replace(a + 1);
}