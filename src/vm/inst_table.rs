use super::fpb::fb2int;
use super::instruction::Instruction;
use crate::api::LuaVM;

const LFIELDS_PER_FLUSH: isize = 50;

/*            NEWTABLE instruction
            R(A) := {} (size = B, C)
    +---------+---------+---------+---------+
    |   B: 0  |   C: 2  |   A: 3  |NEW_TABLE|
    +----|----+-----|---+---------+---------+
         +-------+  +-+
                 |    |       +---------+
    CreateTable(nArr,nRec)--->|    {}   | <-A
                              +---------+
                              |    c    |
                              +---------+
                              |    b    |
                              +---------+
                              |    a    |
                              +---------+
                               registers
*/

pub fn new_table(i: u32, vm: &mut dyn LuaVM) {
    let (a, b, c) = i.abc();
    vm.create_table(fb2int(b as usize), fb2int(c as usize));
    vm.replace(a + 1);
}

/*          GET_TABLE Instruction
            R(A) := R(B)[RK(C)]
    1. Get table with register:
    +---------+---------+---------+---------+
    |   B: 1  | C:0x002 |   A: 3  |GET_TABLE|
    +---------+---------+---------+---------+

        +---------+           +---------+
        |    v    |     +---->|  t[k]   | <-A
        +---------+     |     +---------+
    C-> |    k    |---{[]}    |    c    |
        +---------+     |     +---------+
    B-> |    t    |-----+     |    b    |
        +---------+           +---------+
        |    a    |           |    a    |
        +---------+           +---------+
         registers             registers

    2. Get table with constants table:
    +---------+---------+---------+---------+
    |   B: 1  | C:0x002 |   A:3   |GET_TABLE|
    +---------+---------+---------+---------+

        +---------+           +---------+
    C-> |   100   |---+       |    e    |
        +---------+   |       +---------+
         constants    |   +-->| t[100]  | <-A
                      |   |   +---------+
        |^^^^^^^^^| {[]}--+   |    c    |
        +---------+   |       +---------+
    B-> |    t    |---+       |    t    |
        +---------+           +---------+
        |    a    |           |    a    |
        +---------+           +---------+
         registers             registers
*/
pub fn get_table(i: u32, vm: &mut dyn LuaVM) {
    let (a, b, c) = i.abc();
    let a = a + 1;
    let b = b + 1;
    vm.get_rk(c);
    vm.get_table(b);
    vm.replace(a);
}

/*          SET_TABLE Instruction
            R(A)[RK(B)] = RK[C]
    1. Get table with register:
    +---------+---------+---------+---------+
    | B: 0x002| C:0x003 |   A: 1  |SET_TABLE|
    +---------+---------+---------+---------+

        +---------+                      
    C-> |    v    |-------------+
        +---------+             |        
    B-> |    k    |--------+    |          
        +---------+        |    |        
    A-> |    t    |----->t[k] = v                  
        +---------+                      
        |    a    |                      
        +---------+                      
         registers                      
*/
pub fn set_table(i: u32, vm: &mut dyn LuaVM) {
    let (a, b, c) = i.abc();
    vm.get_rk(b);
    vm.get_rk(c);
    vm.set_table(a + 1);
}

/*          SET_LIST Instruction
    R(A)[(C-1)*FPF+i] := R(A+i), 1 <= i <= B
    1. Get table with register:
    +---------+---------+---------+---------+
    |   B: 3  |   C: 1  |   A: 0  | SET_LIST|
    +---------+---------+---------+---------+

        +---------+     
  A+B-> |    3    |-------+
        +---------+       |   +---------+
  A+2-> |    2    |----+  +-->|    3    | <- (C-1)*50+B
        +---------+    |      +---------+
  A+1-> |    1    |--+ +----->|    2    | <- (C-1)*50+2
        +---------+  |        +---------+
    A-> |    t    |  +------->|    1    | <- (C-1)*50+1
        +---------+           +---------+
         registers                 t
*/
pub fn set_list(i: u32, vm: &mut dyn LuaVM) {
    let (a, b, c) = i.abc();
    let a = a + 1;
    let batch = if c > 0 { c - 1 } else { vm.fetch().ax() };
    let idx = (batch * LFIELDS_PER_FLUSH) as i64;
    for j in 1..=b {
        vm.push_value(a + j);
        vm.set_i(a, idx + j as i64);
    }
}
