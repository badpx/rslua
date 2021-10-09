use std::str;
use byteorder::{ByteOrder, LittleEndian};

use crate::chunk::binary_chunk;

pub struct StreamChunkReader<'a> {
    pub data:   &'a [u8],
}

impl<'a> StreamChunkReader<'a> {
    pub fn read_byte(&mut self) -> u8 {
        let b = self.data[0];
        self.data = &self.data[1..];
        b
    }

    pub fn read_u32(&mut self) -> u32 {
        let _u32 = LittleEndian::read_u32(self.data);
        self.data = &self.data[4..];
        _u32
    }

    pub fn read_u64(&mut self) -> u64 {
        let _u64 = LittleEndian::read_u64(self.data);
        self.data = &self.data[8..];
        _u64
    }

    pub fn read_string(&mut self) -> String {
        let mut size: usize = self.read_byte() as usize;

        if size == 0 {  // NULL
            return String::from("");
        }
        if size == 0xFF {   // Long string
            size = self.read_u64() as usize;
        }
        let bytes = self.read_bytes(size - 1);
        return match str::from_utf8(bytes) {
            Ok(s) => String::from(s),
            Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
        };
    }

    pub fn read_bytes(&mut self, n: usize) -> &[u8] {
        let bytes = &self.data[..n];
        self.data = &self.data[n..];
        bytes
    }

    fn read_lua_integer(&mut self) -> i64 {
        let _i64 = LittleEndian::read_i64(self.data);
        self.data = &self.data[8..];
        _i64
    }

    fn read_lua_number(&mut self) -> f64 {
        let _f64 = LittleEndian::read_f64(self.data);
        self.data = &self.data[8..];
        _f64
    }

    pub fn check_header(&mut self) {
        if self.read_bytes(4) != binary_chunk::LUA_SIGNATURE {
            panic!("Not a precompiled chunk!");
        } else if self.read_byte() != binary_chunk::LUAC_VERSION {
            panic!("Version mismatch!");
        } else if self.read_byte() != binary_chunk::LUAC_FORMAT {
            panic!("Format mismatch!");
        } else if self.read_bytes(6) != binary_chunk::LUAC_DATA {
            panic!("Corrupted!");
        } else if self.read_byte() != binary_chunk::CINT_SIZE {
            panic!("int size mismatch!");
        } else if self.read_byte() != binary_chunk::CSIZET_SIZE {
            panic!("size_t size mismatch!");
        } else if self.read_byte() != binary_chunk::INSTRUCTION_SIZE {
            panic!("instruction size mismatch!");
        } else if self.read_byte() != binary_chunk::LUA_INTEGER_SIZE {
            panic!("lua_Integer size mismatch!");
        } else if self.read_byte() != binary_chunk::LUA_NUMBER_SIZE {
            panic!("lua_Number size mismatch!");
        } else if self.read_lua_integer() != binary_chunk::LUAC_INT {
            panic!("Endianness mismatch!");
        } else if self.read_lua_number() != binary_chunk::LUAC_NUM {
            panic!("Float format mismatch!");
        }
    }

    pub fn read_proto(&mut self, parent_source: &String) -> binary_chunk::Prototype {
        let src = self.read_string();
        let source = if src == "" {
            parent_source.clone()
        } else {
            src
        };
        return binary_chunk::Prototype {
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
            upvalue_names: self.read_upvalue_names()
        };
    }

    fn read_code(&mut self) -> Vec<u32> {
        let mut code = vec![0; self.read_u32() as usize];
        for i in code.iter_mut() {
            *i = self.read_u32();
        }
        code
    }

    fn read_constants(&mut self) -> Vec<binary_chunk::Constant> {
        let len = self.read_u32() as usize;
        let mut constants = Vec::with_capacity(len);
        for _ in 0..len {
            constants.push(self.read_constant());
        }
        constants
    }

    fn read_constant(&mut self) -> binary_chunk::Constant {
        match self.read_byte() {
            binary_chunk::TAG_NIL => binary_chunk::Constant::None,
            binary_chunk::TAG_BOOLEAN => binary_chunk::Constant::Boolean(self.read_byte() != 0),
            binary_chunk::TAG_INTEGER => binary_chunk::Constant::Integer(self.read_lua_integer()),
            binary_chunk::TAG_NUMBER => binary_chunk::Constant::Number(self.read_lua_number()),
            binary_chunk::TAG_SHORT_STR | binary_chunk::TAG_LONG_STR => binary_chunk::Constant::String(self.read_string()),
            _ => panic!("Corrupted!")
        }
    }

    fn read_upvalues(&mut self) -> Vec<binary_chunk::Upvalue> {
        let len = self.read_u32() as usize;
        let mut upvalues = Vec::with_capacity(len);
        for _ in 0..len {
            upvalues.push(binary_chunk::Upvalue {instack: self.read_byte(), idx: self.read_byte()});
        }
        upvalues
    }

    fn read_protos(&mut self, parent_source: &String) -> Vec<binary_chunk::Prototype> {
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

    fn read_local_vars(&mut self) -> Vec<binary_chunk::LocVar> {
        let len = self.read_u32() as usize;
        let mut loc_vars = Vec::with_capacity(len);
        for _ in 0..len {
            loc_vars.push(binary_chunk::LocVar{
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

pub fn undump(data: &[u8]) -> binary_chunk::Prototype {
    let mut reader = StreamChunkReader{data};
    reader.check_header();
    reader.read_byte(); // Skip Upvalue size
    return reader.read_proto(&String::from(""));
}