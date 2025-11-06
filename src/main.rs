mod chunks;
use chunks::*;

fn main() {
    let mut chunk = Chunk::new();
    chunk.write_opcode(OpCode::OpReturn.into());
    chunk.disassemble("test chunk");
    chunk.free();



}