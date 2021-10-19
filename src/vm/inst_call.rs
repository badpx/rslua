use crate::api::LuaVM;
use super::instruction::Instruction;
/*
                CLOSURE Instruction
            R(A) := closure(KPROTO[Bx])
    +-------------------+---------+---------+
    |       Bx: 1       |   A:4   | CLOSURE |
    +-------------------+---------+---------+

                              +---------+
                        +---->|    g    | <-A
                        |     +---------+
                        |     |    f    |
                        |     +---------+
                    {closure} |    c    |
        +---------+     |     +---------+
   Bx-> |    g    |-----+     |    b    |
        +---------+           +---------+
        |    f    |           |    a    |
        +---------+           +---------+
        prototypes             registers
*/

pub fn closure(i: u32, vm: &mut LuaVM) {
    let (a, bx) = i.a_bx();
    vm.load_proto(bx as usize);
    vm.replace(a + 1);
}

/*               RETURN instruction
            return R(A), ..., R(A+B-2)
    +---------+---------+---------+---------+
    |   B: 4  |    C:   |   A: 1  |  RETURN |
    +---------+---------+---------+---------+

        +---------+
A+B-2-> |    b    |---->
        +---------+
  A+1-> |    a    |---->
        +---------+
    A-> |    1    |---->
        +---------+
        |    a    |
        +---------+
         registers
*/
pub fn return_(i: u32, vm: &mut LuaVM) {
    let (a, b, _) = i.abc();
    let a = a + 1;
    if b == 1 {
        // No return values
    } else if b > 1 {
        // b-1 return values
        vm.check_stack((b - 1) as usize);
        for i in a..=(a+b-2) {
            vm.push_value(i);
        }
    } else {
        fix_stack(a, vm);
    }
}
/*             VARARG instruction
        R(A), R(A+1), ..., R(A+B-2) = varag
    +---------+---------+---------+---------+
    |   B: 3  |    C:   |   A: 0  |  VARARG |
    +---------+---------+---------+---------+

        +- - - - -+           +---------+
        |   nil   |---------->|   nil   | <-A+B-2
        +- - - - -+           +---------+
        |    3    |---------->|    3    | <-A+2
        +---------+           +---------+
        |    2    |---------->|    2    | <-A+1
        +---------+           +---------+
        |    1    |---------->|    1    | <-A
        +---------+           +---------+
          varargs              registers
*/
pub fn vararg(i: u32, vm: &mut LuaVM) {
    let (a, b, _) = i.abc();
    if b != 1 {
        vm.load_vararg(b - 1);
        pop_results(a + 1, b, vm);
    }
}

/*               CALL instruction
R(A), ..., R(A+C-2) := R(A)(R(A+1), ... R(A+B-1))
    +---------+---------+---------+---------+
    |   B: 4  |   C: 4  |   A: 0  |   CALL  |
    +---------+---------+---------+---------+

        +---------+             +---------+
A+B-1-> |    3    |-------+     |         |
        +---------+       |     +---------+
  A+2-> |    2    |-----+ |  +->|    c    | <-A+C-2
        +---------+     | |  |  +---------+
  A+1-> |    1    |---+ | |  +->|    b    | <-A+1
        +---------+ f(1,2,3)-+  +---------+
    A-> |    f    |-+      +--->|    a    | <-A
        +---------+             +---------+
         registers               registers
*/
pub fn call(i: u32, vm: &mut LuaVM) {
    let (a, b, c) = i.abc();
    let a = a + 1;
    let nargs = push_func_and_args(a, b, vm);
    vm.call(nargs, c - 1);
    pop_results(a, c, vm);
}

// return R(A)(R(A+1), ..., R(A+B-1))
pub fn tail_call(i: u32, vm: &mut LuaVM) {
    let (a, b, _) = i.abc();
    let a = a + 1;
    let nargs = push_func_and_args(a, b, vm);
    vm.call(nargs, -1);
    pop_results(a, 0, vm);
}

fn pop_results(a: isize, c: isize, vm: &mut LuaVM) {
    if c == 1 {
        // No results
    } else if c > 1 {
        for i in (a..(a + c - 1)).rev() {
            vm.replace(i);
        }
    } else {
        // Leave results on stack
        vm.check_stack(1);
        vm.push_integer(a as i64);
    }
}

fn push_func_and_args(a: isize, b: isize, vm: &mut LuaVM) -> usize {
    if b >= 1 { //b - 1 args
        vm.check_stack(b as usize);
        for i in a..(a+b) {
            vm.push_value(i);
        }
        return b as usize - 1;
    } else {
        fix_stack(a, vm);
        vm.get_top() as usize - vm.register_count() - 1
    }
}

fn fix_stack(a: isize, vm: &mut LuaVM) {
    let x = vm.to_integer(-1) as isize;
    vm.pop(1);

    vm.check_stack((x - a) as usize);
    for i in a..x {
        vm.push_value(i);
    }
    vm.rotate(vm.register_count() as isize + 1, x - a);
}

/*
                SELF Instruction
        R(A+1) := R(B); R(A) := R(B)[RK(C)]
    +---------+---------+---------+---------+
    |   B: 1  | C:0x100 |   A:2   |   SELF  |
    +---------+---------+---------+---------+

        +---------+           +---------+
    C-> |   "f"   |----+      |         |
        +---------+    |      +---------+
         constants  +--+----->|   obj   | <-A+1
                    |  |      +---------+
        |^^^^^^^^^| |  |   +->|  obj.f  | <-A
        +---------+ |  |   |  +---------+
    B-> |   obj   |-+-{[]}-+  |   obj   |
        +---------+           +---------+
        |    a    |           |    a    |
        +---------+           +---------+
         registers             registers
*/
pub fn self_(i: u32, vm: &mut LuaVM) {
    let (a, b, c) = i.abc();
    let a = a + 1;
    let b = b + 1;
    vm.copy(b, a + 1);
    vm.get_rk(c);
    vm.get_table(b);
    vm.replace(a);
}