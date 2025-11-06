mod chunks;
use chunks::*;
mod value;

fn main() {
    let mut chunk = Chunk::new();

    let constant = chunk.add_constant(1.2);
    chunk.write_opcode(OpCode::OpConstant.into(), 123);
    chunk.write(constant, 123);

    chunk.write_opcode(OpCode::OpReturn.into(), 123);
    chunk.disassemble("test chunk");
    chunk.free();



}