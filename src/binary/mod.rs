pub mod chunk;
pub mod reader;

pub fn undump(data: Vec<u8>) -> chunk::Prototype {
    let mut reader = reader::Reader::new(data);
    reader.check_header();
    reader.read_byte(); // Skip Upvalue size
    reader.read_proto()
}
