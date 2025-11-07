mod vm;
use vm::*;
mod chunks;
use chunks::*;
mod value;

fn main() {
    let mut vm = VM::new();

    let mut chunk = Chunk::new();

    let constant = chunk.add_constant(1.0);
    chunk.write_opcode(OpCode::OpConstant.into(), 123);
    chunk.write(constant, 123);

    let constant = chunk.add_constant(3.0);
    chunk.write_opcode(OpCode::OpConstant.into(), 123);
    chunk.write(constant, 123);

    chunk.write_opcode(OpCode::OpMultiply, 123);

    chunk.write_opcode(OpCode::OpNegate, 123);
    chunk.write_opcode(OpCode::OpReturn.into(), 123);
    chunk.disassemble("test chunk");

    vm.interpret(&chunk);

    chunk.free();
    vm.free();
}