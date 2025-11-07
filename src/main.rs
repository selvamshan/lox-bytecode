mod vm;
use vm::*;
mod chunks;
use chunks::*;
mod value;

fn main() {
    let mut vm = VM::new();

    let mut chunk = Chunk::new();

    let constant = chunk.add_constant(1.0);
    chunk.write_opcode(OpCode::Constant.into(), 123);
    chunk.write(constant, 123);

    let constant = chunk.add_constant(3.0);
    chunk.write_opcode(OpCode::Constant.into(), 123);
    chunk.write(constant, 123);

    chunk.write_opcode(OpCode::Multiply, 123);

    chunk.write_opcode(OpCode::Negate, 123);
    chunk.write_opcode(OpCode::Return.into(), 123);
    chunk.disassemble("test chunk");

    vm.interpret(&chunk);

    chunk.free();
    vm.free();
}