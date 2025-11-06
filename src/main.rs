mod chunks;
use chunks::*;
mod value;

fn main() {
    let mut chunk = Chunk::new();
    
    let constant = chunk.add_constant(1.2);
    chunk.write_opcode(OpCode::OpConstant.into());
    chunk.write(constant);

    chunk.write_opcode(OpCode::OpReturn.into());
    chunk.disassemble("test chunk");
    chunk.free();



}