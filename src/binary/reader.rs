use crate::binary::chunk;

pub struct Reader {
    data: Vec<u8>,
    cursor: usize,
}

impl Reader {
    pub fn new(data: Vec<u8>) -> Reader {
        Reader { data: data, cursor: 0 }
    }

    pub fn read_byte(&mut self) -> u8 {
        let b = self.data[self.cursor];
        self.cursor += 1;
        b
    }

    pub fn read_u32(&mut self) -> u32 {
        let a0 = self.read_byte() as u32;
        let a1 = self.read_byte() as u32;
        let a2 = self.read_byte() as u32;
        let a3 = self.read_byte() as u32;
        (a3 << 24) | (a2 << 16) | (a1 << 8) | a0
    }

    pub fn read_u64(&mut self) -> u64 {
        let a0 = self.read_u32() as u64;
        let a1 = self.read_u32() as u64;
        (a1 << 32) | a0
    }

    pub fn read_lua_integer(&mut self) -> i64 {
        self.read_u64() as i64
    }

    pub fn read_lua_number(&mut self) -> f64 {
        use std::f64;
        f64::from_bits(self.read_u64())
    }

    pub fn read_bytes(&mut self, n: usize) -> Vec<u8> {
        let mut vec = Vec::with_capacity(n);
        for _ in 0..n {
            vec.push(self.read_byte());
        }
        vec
    }

    pub fn read_string(&mut self) -> String {
        self._read_string().unwrap_or_else(|| String::new())
    }

    fn _read_string(&mut self) -> Option<String> {
        let mut size: usize = self.read_byte() as usize;

        if size == 0 {
            // NULL
            return None;
        }
        if size == 0xFF {
            // Long string
            size = self.read_u64() as usize;
        }
        let bytes = self.read_bytes(size - 1);
        let string = String::from_utf8(bytes);
        string.ok()
    }

    pub fn check_header(&mut self) {
        assert_eq!(self.read_bytes(4), chunk::LUA_SIGNATURE, "not a precompiled chunk!");
        assert_eq!(self.read_byte(), chunk::LUAC_VERSION, "version mismatch!");
        assert_eq!(self.read_byte(), chunk::LUAC_FORMAT, "format mismatch!");
        assert_eq!(self.read_bytes(6), chunk::LUAC_DATA, "corrupted!");
        assert_eq!(self.read_byte(), chunk::CINT_SIZE, "int size mismatch!");
        assert_eq!(self.read_byte(), chunk::CSIZET_SIZE, "size_t size mismatch!");
        assert_eq!(self.read_byte(), chunk::INSTRUCTION_SIZE, "instruction size mismatch!");
        assert_eq!(self.read_byte(), chunk::LUA_INTEGER_SIZE, "lua_Integer size mismatch!");
        assert_eq!(self.read_byte(), chunk::LUA_NUMBER_SIZE, "lua_Number size mismatch!");
        assert_eq!(self.read_lua_integer(), chunk::LUAC_INT, "endianness mismatch!");
        assert_eq!(self.read_lua_number(), chunk::LUAC_NUM, "float format mismatch!");
    }

    pub fn read_proto(&mut self) -> chunk::Prototype {
        self._read_proto(None)
    }

    fn _read_proto(&mut self, parent_source: Option<String>) -> chunk::Prototype {
        let source = self._read_string().or(parent_source);
        return chunk::Prototype {
            source: source.clone(),
            line_defined: self.read_u32(),
            last_line_defined: self.read_u32(),
            num_params: self.read_byte(),
            is_vararg: self.read_byte(),
            max_stack_size: self.read_byte(),
            code: self.read_vec(|r| r.read_u32()),
            constants: self.read_vec(|r| r.read_constant()),
            upvalues: self.read_vec(|r| r.read_upvalue()),
            protos: self.read_vec(|r| r._read_proto(source.clone())),
            line_info: self.read_vec(|r| r.read_u32()),
            loc_vars: self.read_vec(|r| r.read_loc_var()),
            upvalue_names: self.read_vec(|r| r.read_string()),
        };
    }

    // A template for read vector
    fn read_vec<F, T>(&mut self, func: F) -> Vec<T>
    where
        F: Fn(&mut Reader) -> T,
    {
        let n = self.read_u32() as usize;
        let mut vec = Vec::with_capacity(n);
        for _ in 0..n {
            vec.push(func(self));
        }
        vec
    }

    fn read_constant(&mut self) -> chunk::Constant {
        match self.read_byte() {
            chunk::TAG_NIL => chunk::Constant::Nil,
            chunk::TAG_BOOLEAN => chunk::Constant::Boolean(self.read_byte() != 0),
            chunk::TAG_INTEGER => chunk::Constant::Integer(self.read_lua_integer()),
            chunk::TAG_NUMBER => chunk::Constant::Number(self.read_lua_number()),
            chunk::TAG_SHORT_STR | chunk::TAG_LONG_STR => chunk::Constant::String(self.read_string()),
            _ => panic!("Corrupted!"),
        }
    }

    fn read_upvalue(&mut self) -> chunk::Upvalue {
        chunk::Upvalue {
            instack: self.read_byte(),
            idx: self.read_byte(),
        }
    }

    fn read_loc_var(&mut self) -> chunk::LocVar {
        chunk::LocVar {
            var_name: self.read_string(),
            start_pc: self.read_u32(),
            end_pc: self.read_u32(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_header() {
        let lua5_3_header = vec![
            0x1bu8, 0x4c, 0x75, 0x61, 0x53, 0x00, 0x19, 0x93, 0x0d, 0x0a, 0x1a, 0x0a,
            0x04, 0x08, 0x04, 0x08, 0x08, 0x78, 0x56, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x28, 0x77, 0x40,
          ];
        let mut reader = Reader::new(lua5_3_header);
        reader.check_header();
    }
}