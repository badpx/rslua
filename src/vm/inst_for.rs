use crate::api::consts::*;
use crate::api::LuaVM;
use super::instruction::Instruction;

/*              FORPREP instruction
            R(A) -= R(A+2); pc += sBx
    +-------------------+---------+---------+
    |      sBx: 2       |   A: 0  | FORPREP |
    +-------------------+---------+---------+

        +----------+          +----------+
  A+3-> |    i:    |          |    i:    |
        +----------+          +----------+
  A+2-> |(step): 2 |---+      |(step): 2 |
        +----------+   |      +----------+
  A+1-> |(limit):10|  {-}--+  |(limit):10|
        +----------+   |   |  +----------+
    A-> |(index):1 |---+   +->|(index):-1|
        +----------+          +----------+
         registers             registers
*/
pub fn for_prep(i: u32, vm: &mut LuaVM) {
    let (a, sbx) = i.a_sbx();
    let a = a + 1;

    for o in 0..=2 {
        if vm.type_id(a + o) == LUA_TSTRING {
            vm.push_number(vm.to_number(a + o));
            vm.replace(a + o);
        }
    }
    // R(A) -= R(A+2)
    vm.push_value(a);
    vm.push_value(a + 2);
    vm.arith(LUA_OPSUB);
    vm.replace(a);
    // pc += sBx
    vm.add_pc(sbx);
}

/*              FORLOOP instruction
            R(A) += R(A+2);
            if R(A) <?= R(A+1) then {
                pc += sBx; R(A+3)=R(A)
            }
    +-------------------+---------+---------+
    |      sBx: -3      |   A: 0  | FORLOOP |
    +-------------------+---------+---------+

        +----------+          +----------+
  A+3-> |    i:    |          |    i: 1  |<-+
        +----------+          +----------+  |
  A+2-> |(step): 2 |---+      |(step): 2 |  |
        +----------+   |      +----------+  |
  A+1-> |(limit):10|  {+}--+  |(limit):10|  |
        +----------+   |   |  +----------+  |
    A-> |(index):-1|---+   +->|(index):1 |--+
        +----------+          +----------+
         registers             registers
*/
pub fn for_loop(i: u32, vm: &mut LuaVM) {
    let (mut a, sbx) = i.a_sbx();
    a += 1;

    // R(A)+=R(A+2);
    vm.push_value(a + 2);
    vm.push_value(a);
    vm.arith(LUA_OPADD);
    vm.replace(a);

    let positive_step = vm.to_number(a + 2) >= 0.0;
    if positive_step && vm.compare(a, a + 1, LUA_OPLE) || !positive_step && vm.compare(a + 1, a, LUA_OPLE) {
        // pc+=sBx; R(A+3)=R(A)
        vm.add_pc(sbx);
        vm.copy(a, a + 3);
    }
}
/*
pub fn for_loop(i: u32, vm: &mut LuaVM) {
    let (a, sbx) = i.a_sbx();
    let a = a + 1;
    // R(A) += R(A+2)
    vm.push_value(a + 2);
    vm.push_value(a);
    vm.arith(LUA_OPADD);
    vm.replace(a);
    // R(A) <?= R(A+1) [if step is positive, `<?=` means `<=`, else means `>=`]
    let is_positive_step = vm.to_number(a + 2) >= 0.0;
    if  is_positive_step && vm.compare(a, a + 1, LUA_OPLE) || !is_positive_step && vm.compare(a + 1, a, LUA_OPLE) {
            vm.add_pc(sbx);     // pc += sBx
            vm.copy(a, a + 3);  // R(A+3) = R(A)
        }
}
*/