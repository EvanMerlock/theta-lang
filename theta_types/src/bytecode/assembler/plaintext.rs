use crate::bytecode::{Chunk, OpCode, ThetaBitstream, ThetaFunction, ThetaConstant};

use super::{AssembleError, Assembler};

pub struct PlainTextAssembler<'a> {
    output_file: &'a mut Box<dyn std::io::Write>,
}

impl<'a> PlainTextAssembler<'a> {
    pub fn new(file_out: &'a mut Box<dyn std::io::Write>) -> PlainTextAssembler {
        PlainTextAssembler {
            output_file: file_out,
        }
    }
}

impl<'a> Assembler for PlainTextAssembler<'a> {
    type Out = Result<(), AssembleError>;

    fn assemble(&mut self, _bitstream: ThetaBitstream) -> Result<(), AssembleError> {
        todo!()
    }

    fn assemble_bitstream(&mut self, _bitstream: ThetaBitstream) -> Self::Out {
        todo!()
    }

    fn assemble_chunk(&mut self, chunk: Chunk) -> Self::Out {
        writeln!(self.output_file, "=== CHUNK BEGIN ===")?;


        writeln!(self.output_file, "-- INSTRUCTIONS --")?;

        let mut code_offset: usize = 0;
        let instructions_in_chunk = chunk.instructions();
        for opcode in instructions_in_chunk {
            write!(
                self.output_file,
                "{code_offset:#X} | Op: {} ({:#X})",
                opcode.human_readable(),
                opcode.as_hexcode()
            )?;

            match opcode {
                OpCode::JumpFar { offset } => {
                    write!(self.output_file, " -> {:#X}", code_offset as isize + offset)?
                }
                OpCode::JumpLocal { offset } => write!(
                    self.output_file,
                    " -> {:#X}",
                    code_offset as isize + *offset as isize
                )?,
                OpCode::JumpFarIfFalse { offset } => {
                    write!(self.output_file, " -> {:#X}", code_offset as isize + offset)?
                }
                OpCode::JumpLocalIfFalse { offset } => write!(
                    self.output_file,
                    " -> {:#X}",
                    code_offset as isize + *offset as isize
                )?,
                _ => {}
            }

            writeln!(self.output_file)?;

            code_offset += opcode.size();
        }
        writeln!(self.output_file, "=== CHUNK END @ {code_offset:#X} ===")?;
        Ok(())
    }

    fn assemble_constant_pool(&mut self, constant_pool: Vec<ThetaConstant>) -> Self::Out {
        writeln!(self.output_file, "-- CONSTANT POOL --")?;

        for constant in constant_pool {
            writeln!(self.output_file, "Constant: {:?}", constant)?;
        }

        Ok(())
    }

    fn assemble_function_pool(&mut self, _function_pool: Vec<ThetaFunction>) -> Self::Out {
        todo!()
    }
}
