use std::str;

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

    fn read_lua_integer(&mut self) -> i64 {
        self.read_u64() as i64
    }

    fn read_lua_number(&mut self) -> f64 {
        use std::f64;
        f64::from_bits(self.read_u64())
    }

    pub fn read_string(&mut self) -> String {
        let mut size: usize = self.read_byte() as usize;

        if size == 0 {
            // NULL
            return String::new();
        }
        if size == 0xFF {
            // Long string
            size = self.read_u64() as usize;
        }
        let bytes = self.read_bytes(size - 1);
        match String::from_utf8(bytes) {
            Ok(s) => String::from(s),
            Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
        }
    }

    pub fn read_bytes(&mut self, n: usize) -> Vec<u8> {
        let mut vec = Vec::with_capacity(n);
        for _ in 0..n {
            vec.push(self.read_byte());
        }
        vec
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

    pub fn read_proto(&mut self, parent_source: &String) -> chunk::Prototype {
        let src = self.read_string();
        let source = if src == "" {
            parent_source.clone()
        } else {
            src
        };
        return chunk::Prototype {
            source: source.clone(),
            line_defined: self.read_u32(),
            last_line_defined: self.read_u32(),
            num_params: self.read_byte(),
            is_vararg: self.read_byte(),
            max_stack_size: self.read_byte(),
            code: self.read_code(),
            constants: self.read_constants(),
            upvalues: self.read_upvalues(),
            protos: self.read_protos(&source),
            line_info: self.read_line_info(),
            loc_vars: self.read_local_vars(),
            upvalue_names: self.read_upvalue_names(),
        };
    }

    fn read_code(&mut self) -> Vec<u32> {
        let mut code = vec![0; self.read_u32() as usize];
        for i in code.iter_mut() {
            *i = self.read_u32();
        }
        code
    }

    fn read_constants(&mut self) -> Vec<chunk::Constant> {
        let len = self.read_u32() as usize;
        let mut constants = Vec::with_capacity(len);
        for _ in 0..len {
            constants.push(self.read_constant());
        }
        constants
    }

    fn read_constant(&mut self) -> chunk::Constant {
        match self.read_byte() {
            chunk::TAG_NIL => chunk::Constant::Nil,
            chunk::TAG_BOOLEAN => chunk::Constant::Boolean(self.read_byte() != 0),
            chunk::TAG_INTEGER => chunk::Constant::Integer(self.read_lua_integer()),
            chunk::TAG_NUMBER => chunk::Constant::Number(self.read_lua_number()),
            chunk::TAG_SHORT_STR | chunk::TAG_LONG_STR => {
                chunk::Constant::String(self.read_string())
            }
            _ => panic!("Corrupted!"),
        }
    }

    fn read_upvalues(&mut self) -> Vec<chunk::Upvalue> {
        let len = self.read_u32() as usize;
        let mut upvalues = Vec::with_capacity(len);
        for _ in 0..len {
            upvalues.push(chunk::Upvalue {
                instack: self.read_byte(),
                idx: self.read_byte(),
            });
        }
        upvalues
    }

    fn read_protos(&mut self, parent_source: &String) -> Vec<chunk::Prototype> {
        let len = self.read_u32() as usize;
        let mut protos = Vec::with_capacity(len);
        for _ in 0..len {
            protos.push(self.read_proto(&parent_source));
        }
        protos
    }

    fn read_line_info(&mut self) -> Vec<u32> {
        let mut line_info = vec![0; self.read_u32() as usize];
        for e in line_info.iter_mut() {
            *e = self.read_u32();
        }
        line_info
    }

    fn read_local_vars(&mut self) -> Vec<chunk::LocVar> {
        let len = self.read_u32() as usize;
        let mut loc_vars = Vec::with_capacity(len);
        for _ in 0..len {
            loc_vars.push(chunk::LocVar {
                var_name: self.read_string(),
                start_pc: self.read_u32(),
                end_pc: self.read_u32(),
            });
        }
        loc_vars
    }

    fn read_upvalue_names(&mut self) -> Vec<String> {
        let len = self.read_u32() as usize;
        let mut names = Vec::with_capacity(len);
        for _ in 0..len {
            names.push(self.read_string());
        }
        names
    }
}
