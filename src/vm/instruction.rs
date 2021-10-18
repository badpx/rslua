use super::inst_for::*;
use super::inst_load::*;
use super::inst_misc::*;
use super::inst_ops::*;
use super::inst_table::*;
use super::opcodes::*;
use crate::api::LuaVM;


const MAXARG_BX: isize = (1 << 18) - 1; // 262143
const MAXARG_SBX: isize = MAXARG_BX >> 1; // 131071

/*
        31       22       13       5    0
         +-------+^------+-^-----+-^-----
  iABC:  |b=9bits |c=9bits |a=8bits|op=6|
         +-------+^------+-^-----+-^-----
  iABx:  |    bx=18bits    |a=8bits|op=6|
         +-------+^------+-^-----+-^-----
  iAsBx: |   sbx=18bits    |a=8bits|op=6|
         +-------+^------+-^-----+-^-----
  iAx:   |    ax=26bits            |op=6|
         +-------+^------+-^-----+-^-----
        31      23      15       7      0
*/
pub trait Instruction {
    fn opname(self) -> &'static str;
    fn opmode(self) -> u8;
    fn b_mode(self) -> u8;
    fn c_mode(self) -> u8;
    fn opcode(self) -> u8;
    fn abc(self) -> (isize, isize, isize);
    fn a_bx(self) -> (isize, isize);
    fn a_sbx(self) -> (isize, isize);
    fn ax(self) -> isize;
    fn execute(self, vm: &mut dyn LuaVM);
}

impl Instruction for u32 {
    fn opname(self) -> &'static str {
        OPCODES[self.opcode() as usize].name
    }

    fn opmode(self) -> u8 {
        OPCODES[self.opcode() as usize].opmode
    }

    fn b_mode(self) -> u8 {
        OPCODES[self.opcode() as usize].bmode
    }

    fn c_mode(self) -> u8 {
        OPCODES[self.opcode() as usize].cmode
    }

    fn opcode(self) -> u8 {
        self as u8 & 0x3F
    }

    fn abc(self) -> (isize, isize, isize) {
        let a = (self >> 6 & 0xFF) as isize;
        let c = (self >> 14 & 0x1FF) as isize;
        let b = (self >> 23 & 0x1FF) as isize;
        (a, b, c)
    }

    /*
            0               131071             262143
        Bx  |------------------+------------------|


          -131071              0               131072
       sBx  |------------------+------------------|
    */

    fn a_bx(self) -> (isize, isize) {
        let a = (self >> 6 & 0xFF) as isize;
        let bx = (self >> 14) as isize;
        (a, bx)
    }

    fn a_sbx(self) -> (isize, isize) {
        let (a, bx) = self.a_bx();
        (a, bx - MAXARG_SBX)
    }

    fn ax(self) -> isize {
        (self >> 6) as isize
    }

    fn execute(self, vm: &mut dyn LuaVM) {
        match self.opcode() {
            OP_MOVE => move_(self, vm),
            OP_LOADK => load_k(self, vm),
            OP_LOADKX => load_kx(self, vm),
            OP_LOADBOOL => load_bool(self, vm),
            OP_LOADNIL => load_nil(self, vm),
            // OP_GETUPVAL => (),
            // OP_GETTABUP => (),
            OP_GETTABLE => get_table(self, vm),
            // OP_SETTABUP => (),
            // OP_SETUPVAL => (),
            OP_SETTABLE => set_table(self, vm),
            OP_NEWTABLE => new_table(self, vm),
            // OP_SELF => (),
            OP_ADD => add(self, vm),
            OP_SUB => sub(self, vm),
            OP_MUL => mul(self, vm),
            OP_MOD => mod_(self, vm),
            OP_POW => pow(self, vm),
            OP_DIV => div(self, vm),
            OP_IDIV => idiv(self, vm),
            OP_BAND => band(self, vm),
            OP_BOR => bor(self, vm),
            OP_BXOR => bxor(self, vm),
            OP_SHL => bshl(self, vm),
            OP_SHR => bshr(self, vm),
            OP_UNM => unm(self, vm),
            OP_BNOT => bnot(self, vm),
            OP_NOT => not(self, vm),
            OP_LEN => length(self, vm),
            OP_CONCAT => concat(self, vm),
            OP_JMP => jmp(self, vm),
            OP_EQ => eq(self, vm),
            OP_LT => lt(self, vm),
            OP_LE => le(self, vm),
            OP_TEST => test(self, vm),
            OP_TESTSET => test_set(self, vm),
            // OP_CALL => (),
            // OP_TAILCALL => (),
            // OP_RETURN => (),
            OP_FORLOOP => for_loop(self, vm),
            OP_FORPREP => for_prep(self, vm),
            // OP_TFORCALL => (),
            // OP_TFORLOOP => (),
            OP_SETLIST => set_list(self, vm),
            // OP_CLOSURE => (),
            // OP_VARARG => (),
            // OP_EXTRAARG => (),
            _ => {
                dbg!(self.opname());
                unimplemented!()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::state;
    use crate::binary::{chunk, undump};
    use crate::api::LuaAPI;
    use crate::LuaState;
    use crate::binary::reader::tests::LUA_FOR_LOOP;
    use super::*;

    /* Lua source code:
        local t = {"a", "b", "c"}
        t[2] = "B"
        t["foo"] = "Bar"
        local s = t[3] .. t[2] .. t[1] .. t["foo"] .. #t
    */
    const LUA_TABLE_CHUNK:&[u8] = &[
        0x1b, 0x4c, 0x75, 0x61, 0x53, 0x00, 0x19, 0x93, 0x0d, 0x0a, 0x1a, 0x0a,
        0x04, 0x08, 0x04, 0x08, 0x08, 0x78, 0x56, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x28, 0x77, 0x40, 0x01, 0x0b, 0x40,
        0x74, 0x61, 0x62, 0x6c, 0x65, 0x2e, 0x6c, 0x75, 0x61, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x06, 0x0e, 0x00, 0x00, 0x00,
        0x0b, 0x00, 0x80, 0x01, 0x41, 0x00, 0x00, 0x00, 0x81, 0x40, 0x00, 0x00,
        0xc1, 0x80, 0x00, 0x00, 0x2b, 0x40, 0x80, 0x01, 0x0a, 0x00, 0xc1, 0x81,
        0x0a, 0x80, 0xc1, 0x82, 0x47, 0xc0, 0x41, 0x00, 0x87, 0xc0, 0x40, 0x00,
        0xc7, 0x00, 0x42, 0x00, 0x07, 0x41, 0x41, 0x00, 0x5c, 0x01, 0x00, 0x00,
        0x5d, 0x40, 0x81, 0x00, 0x26, 0x00, 0x80, 0x00, 0x09, 0x00, 0x00, 0x00,
        0x04, 0x02, 0x61, 0x04, 0x02, 0x62, 0x04, 0x02, 0x63, 0x13, 0x02, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x02, 0x42, 0x04, 0x04, 0x66,
        0x6f, 0x6f, 0x04, 0x04, 0x42, 0x61, 0x72, 0x13, 0x03, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x13, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0e,
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x02,
        0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x04,
        0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x04,
        0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x02,
        0x00, 0x00, 0x00, 0x02, 0x74, 0x05, 0x00, 0x00, 0x00, 0x0e, 0x00, 0x00,
        0x00, 0x02, 0x73, 0x0d, 0x00, 0x00, 0x00, 0x0e, 0x00, 0x00, 0x00, 0x01,
        0x00, 0x00, 0x00, 0x05, 0x5f, 0x45, 0x4e, 0x56
    ];

    #[test]
    fn test_forloop() {
        let proto = undump(LUA_FOR_LOOP.to_vec());
        let ls = execute(proto);
        let result = ls.stack().get(1);
        assert_eq!(result.to_integer(), Some(2550));
    }

    #[test]
    fn test_table() {
        let proto = undump(LUA_TABLE_CHUNK.to_vec());
        let ls = execute(proto);
        let result = ls.to_string(2);
        assert_eq!(result, "cBaBar3");
    }

    fn execute(proto: chunk::Prototype) -> LuaState {
        let regs_size = proto.max_stack_size;
        let mut ls = state::new_lua_state((regs_size + 8) as usize, proto);
        ls.set_top(regs_size as isize);
        loop {
            let pc = ls.pc();
            let inst = ls.fetch();
            if inst.opcode() != OP_RETURN {
                inst.execute(&mut ls);
                print!("[{:04}] {} ", pc + 1, inst.opname());
            } else {
                break;
            }
            println!("{:?}", ls.stack()._raw_data());
        }
        ls 
    }
}
