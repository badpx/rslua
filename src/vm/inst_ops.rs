use crate::api::consts::*;
use crate::ArithOp;
use crate::api::LuaVM;
use super::instruction::Instruction;

/*              Binary arith
            R(A) := RK(B) op RK(C)
    +---------+---------+---------+---------+
    | B:0x001 | C:0x100 |   A:4   | BinaryOP|
    +---------+---------+---------+---------+

        +---------+           +---------+
    C-> |   100   |---+   +-->| b op 100| <-A
        +---------+   |   |   +---------+
         constants    |   |   |    d    |
                    {op}--+   +---------+
        |^^^^^^^^^|   |       |    c    |
        +---------+   |       +---------+
    B-> |    b    |---+       |    b    |
        +---------+           +---------+
        |    a    |           |    a    |
        +---------+           +---------+
         registers             registers
*/
fn _binary_arith(i: u32, vm: &mut dyn LuaVM, op: ArithOp) {
    let (a, b, c) = i.abc();    
    vm.get_rk(b);
    vm.get_rk(c);
    vm.arith(op);
    vm.replace(a + 1);
}

/*               unary arith
                R(A) := op R(B)
    +---------+---------+---------+---------+
    |   B: 1  |    C:   |   A: 3  | UnaryOP |
    +---------+---------+---------+---------+

        +---------+           +---------+
        |    d    |     +---->|  op b   | <-A
        +---------+     |     +---------+
        |    c    |    {op}   |    c    |
        +---------+     |     +---------+
        |    b    |-----+     |    b    |
        +---------+           +---------+
    B-> |    a    |           |    a    |
        +---------+           +---------+
         registers             registers
*/
fn _unary_arith(i: u32, vm: &mut dyn LuaVM, op: ArithOp) {
    let (a, b, _) = i.abc();
    vm.push_value(b + 1);
    vm.arith(op);
    vm.replace(a + 1);
}

// arith
pub fn add(i: u32, vm: &mut dyn LuaVM) { _binary_arith(i, vm, LUA_OPADD); } // +
pub fn sub(i: u32, vm: &mut dyn LuaVM) { _binary_arith(i, vm, LUA_OPSUB); } // -
pub fn mul(i: u32, vm: &mut dyn LuaVM) { _binary_arith(i, vm, LUA_OPMUL); } // *
pub fn mod_(i: u32, vm: &mut dyn LuaVM) { _binary_arith(i, vm, LUA_OPMOD); }// %
pub fn pow(i: u32, vm: &mut dyn LuaVM) { _binary_arith(i, vm, LUA_OPPOW); } // ^
pub fn div(i: u32, vm: &mut dyn LuaVM) { _binary_arith(i, vm, LUA_OPDIV); } // /
pub fn idiv(i: u32, vm: &mut dyn LuaVM) { _binary_arith(i, vm, LUA_OPIDIV); }   // //
pub fn band(i: u32, vm: &mut dyn LuaVM) { _binary_arith(i, vm, LUA_OPBAND); }   // &
pub fn bor(i: u32, vm: &mut dyn LuaVM) { _binary_arith(i, vm, LUA_OPBOR); } // |
pub fn bxor(i: u32, vm: &mut dyn LuaVM) { _binary_arith(i, vm, LUA_OPBXOR); }   // ~
pub fn bshl(i: u32, vm: &mut dyn LuaVM) { _binary_arith(i, vm, LUA_OPSHL); }// <<
pub fn bshr(i: u32, vm: &mut dyn LuaVM) { _binary_arith(i, vm, LUA_OPSHR); }// >>
pub fn unm(i: u32, vm: &mut dyn LuaVM) { _binary_arith(i, vm, LUA_OPUNM); } // -
pub fn bnot(i: u32, vm: &mut dyn LuaVM) { _binary_arith(i, vm, LUA_OPBNOT); }   // ~

/*               LEN instruction
            R(A) := length of R(B)
    +---------+---------+---------+---------+
    |   B: 1  |    C:   |   A: 3  |   LEN   |
    +---------+---------+---------+---------+

        +---------+           +---------+
        |    d    |     +---->|   #b    | <-A
        +---------+     |     +---------+
        |    c    |    {#}    |    c    |
        +---------+     |     +---------+
        |    b    |-----+     |    b    |
        +---------+           +---------+
    B-> |    a    |           |    a    |
        +---------+           +---------+
         registers             registers
*/
pub fn length(i: u32, vm: &mut dyn LuaVM) {
    let (a, b, _) = i.abc();
    vm.len(b + 1);
    vm.replace(a + 1)
}

/*              CONCAT instruction
            R(A) := R(B).. ... ..R(C)
    +---------+---------+---------+---------+
    |   B: 1  |   C: 3  |   A: 0  |  CONCAT |
    +---------+---------+---------+---------+

        +---------+           +---------+
    C-> |    d    |-----+     |    b    |
        +---------+     |     +---------+
  B+1-> |    c    |---{..}-+  |    c    |
        +---------+     |  |  +---------+
    B-> |    b    |-----+  |  |    b    |
        +---------+        |  +---------+
        |    a    |        +->| b..c..d | <-A
        +---------+           +---------+
         registers             registers
*/
pub fn concat(i: u32, vm: &mut dyn LuaVM) {
    let (a, b, c) = i.abc();
    let a = a + 1;
    let b = b + 1;
    let c = c + 1;
    let n = c - b + 1;
    vm.check_stack(n as usize);
    for i in b..=c {
        vm.push_value(i);
    }
    vm.concat(n);
    vm.replace(a);
}

/*              Compare Instruction
        if ((RK(B) op RC(C)) ~= A) then pc++
    +---------+---------+---------+---------+
    | B:0x001 | C:0x100 |  A: 1   |  CMP_OP |
    +---------+---------+----+----+---------+
                              |
        +---------+         {bool}
    C-> |  "foo"  |---+       |
        +---------+   |       |
         constants    |       |
                     {op}----{~=}----> JMP 1
        |^^^^^^^^^|   |
        +---------+   |
    B-> |    b    |---+
        +---------+
        |    a    |
        +---------+
         registers
*/
fn _compare(i: u32, vm: &mut dyn LuaVM, op: CompareOp) {
    let (a, b, c) = i.abc();
    vm.get_rk(b);
    vm.get_rk(c);
    if vm.compare(-2, -1, op) != (a != 0) {
        vm.add_pc(1);
    }
    vm.pop(2);
}

/* compare */
pub fn eq(i: u32, vm: &mut dyn LuaVM) { _compare(i, vm, LUA_OPEQ) } // ==
pub fn lt(i: u32, vm: &mut dyn LuaVM) { _compare(i, vm, LUA_OPLT) } // <
pub fn le(i: u32, vm: &mut dyn LuaVM) { _compare(i, vm, LUA_OPLE) } // <=

/* logical */

/*               NOT Instruction
                R(A) := not R(B)
    +---------+---------+---------+---------+
    |   B: 1  |    C:   |   A: 3  |   NOT   |
    +---------+---------+---------+---------+

        +---------+           +---------+
        |    d    |     +---->|  not b  | <-A
        +---------+     |     +---------+
        |    c    |     |     |    c    |
        +---------+     |     +---------+
        |    b    |-----+     |    b    |
        +---------+           +---------+
    B-> |    a    |           |    a    |
        +---------+           +---------+
         registers             registers
*/
pub fn not(i: u32, vm: &mut dyn LuaVM) {
    let (a, b, _) = i.abc();
    vm.push_boolean(!vm.to_boolean(b + 1));
    vm.replace(a + 1);
}

/*            TESTSET instruction
    if (R(B) <=> C) then R(A) := R(B) else pc++
    +----------+---------+---------+--------+
    |   B: 3   |   C: 0  |   A:1   | TESTSET|
    +----------+-----|---+---------+--------+
                     +------------+
                                  |
        +---------+  +--{bool}  {bool}    
        |    e    |  |     |      |
        +---------+  |     +-{==}-+   no
    B-> |    d    |--|        /\____+----> JMP 1
        +---------+  |   yes/   
        |    c    |  |    /   |^^^^^^^^^|
        +---------+  |<--+    +---------+
        |    b    |  +------->|    b    | <-A
        +---------+           +---------+
        |    a    |           |    a    |
        +---------+           +---------+
         registers             registers
*/
pub fn test_set(i: u32, vm: &mut dyn LuaVM) {
    let (a, b, c) = i.abc();
    let a = a + 1;
    let b = b + 1;
    if vm.to_boolean(b) == (c != 0) {
        vm.copy(b, a);
    } else {
        vm.add_pc(1);
    }
}

/*              TEST instruction
            if not (R(A) <=> C) then pc++
    +----------+---------+---------+--------+
    |     B:   |   C: 0  |   A:1   |  TEST  |
    +----------+-----|---+---------+--------+
                     +---+
        +---------+      |
        |    d    |    {bool}---+   no
        +---------+            {==}----> JMP 1
        |    c    |    {bool}---+
        +---------+      |
        |    b    |------+
        +---------+
        |    b    |
        +---------+
         registers 
*/
pub fn test(i: u32, vm: &mut dyn LuaVM) {
    let (a, _, c) = i.abc();
    if vm.to_boolean(a + 1) != (c != 0) {
        vm.add_pc(1);
    }
}