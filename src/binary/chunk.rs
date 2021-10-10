pub const LUA_SIGNATURE: &'static [u8; 4] = b"\x1BLua";
pub const LUAC_VERSION: u8 = 0x53;
pub const LUAC_FORMAT: u8 = 0;
pub const LUAC_DATA: &'static [u8; 6] = b"\x19\x93\r\n\x1A\n";
pub const CINT_SIZE: u8 = 4;
pub const CSIZET_SIZE: u8 = 8;
pub const INSTRUCTION_SIZE: u8 = 4;
pub const LUA_INTEGER_SIZE: u8 = 8;
pub const LUA_NUMBER_SIZE: u8 = 8;
pub const LUAC_INT: i64 = 0x5678;
pub const LUAC_NUM: f64 = 370.5;

pub const TAG_NIL: u8 = 0x00;
pub const TAG_BOOLEAN: u8 = 0x01;
pub const TAG_NUMBER: u8 = 0x03;
pub const TAG_INTEGER: u8 = 0x13;
pub const TAG_SHORT_STR: u8 = 0x04;
pub const TAG_LONG_STR: u8 = 0x14;

#[allow(dead_code)]
pub struct BinaryChunk {
    pub header: Header,
    pub size_upvalues: u8,
    pub main_func: Prototype,
}

#[allow(dead_code)]
pub struct Header {
    pub signature: [u8; 4],
    pub version: i8,
    pub format: i8,
    pub luac_data: [u8; 6],
    pub cint_size: u8,
    pub sizet_size: u8,
    pub instruction_size: u8,
    pub lua_integer_size: u8,
    pub lua_number_size: u8,
    pub luac_int: i64,
    pub luac_num: f64,
}

pub struct Prototype {
    pub source: String,
    pub line_defined: u32,
    pub last_line_defined: u32,
    pub num_params: u8,
    pub is_vararg: u8,
    pub max_stack_size: u8,
    pub code: Vec<u32>,
    pub constants: Vec<Constant>,
    pub upvalues: Vec<Upvalue>,
    pub protos: Vec<Prototype>,
    pub line_info: Vec<u32>,
    pub loc_vars: Vec<LocVar>,
    pub upvalue_names: Vec<String>,
}

pub enum Constant {
    Nil,
    Boolean(bool),
    Integer(i64),
    Number(f64),
    String(String),
}

pub struct Upvalue {
    pub instack: u8,
    pub idx: u8,
}

pub struct LocVar {
    pub var_name: String,
    pub start_pc: u32,
    pub end_pc: u32,
}
