use clap::{clap_derive::ArgEnum, Parser as ClapParser};
use log::debug;
use theta_compiler::{lexer::{BasicLexer, Lexer}, ast::{symbol::SymbolTable, transformers::{typeck::TypeCk, to_bytecode::ToByteCode, ASTTransformer}}, parser::{BasicParser, Parser}};
use theta_types::bytecode::{Assembler, AssembleError, BasicAssembler, PlainTextAssembler};
use std::{fs::File, io::BufReader, cell::RefCell, rc::Rc};
use theta::repl::parser::{ReplParser, ReplItem};

#[derive(ClapParser)]
#[clap(version = "0.0.1", author = "Evan Merlock")]
struct ThetaCOptions {
    #[clap(short, long)]
    in_file: Option<String>,
    #[clap(short, long)]
    out_file: Option<String>,
    #[clap(short, long, parse(from_occurrences))]
    verbose: i32,
    #[clap(arg_enum)]
    assembler: AssemblerImpl,
}

#[derive(Clone, ArgEnum)]
enum AssemblerImpl {
    Basic,
    String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let options = ThetaCOptions::parse();

    let mut in_file: Box<dyn std::io::BufRead> = {
        if options.in_file.is_some() {
            Box::new(BufReader::new(File::open(options.in_file.unwrap())?))
        } else {
            Box::new(BufReader::new(std::io::stdin()))
        }
    };

    let mut out_file: Box<dyn std::io::Write> = {
        if options.out_file.is_some() {
            Box::new(File::create(options.out_file.unwrap())?)
        } else {
            Box::new(std::io::stdout())
        }
    };

    let mut char_buf = Vec::new();
    in_file.read_to_end(&mut char_buf)?;

    let byte_stream = String::from_utf8(char_buf)?;
    let mut characters = byte_stream.chars();

    let lexer = BasicLexer::new(&mut characters);
    let tokens = lexer.lex()?;
    let tbl = Rc::new(RefCell::new(SymbolTable::default()));
    let parser = BasicParser::new_sym(tokens.output(), tbl);
    let parser = ReplParser::new(parser);
    let trees = parser.parse()?;
    let tbc = ToByteCode::new(tokens.line_mapping());
    for pi in trees {
        write!(out_file, "==== NEW ITEM ====\r\n")?;
        match pi {
            ReplItem::ParserItem(item) => {
                let sym = item.information().current_symbol_table.clone();
                debug!("sym: {:?}", sym.borrow());
                let type_cker = TypeCk::new(sym);
                let type_check = type_cker.transform_item(&item)?;
                let thefunc = tbc.transform_item(&type_check)?;
                debug!("item: {:?}", thefunc);
            
                {
                    let mut assembler: Box<dyn Assembler<Out = Result<(), AssembleError>>> =
                        match options.assembler {
                            AssemblerImpl::Basic => Box::new(BasicAssembler::new(&mut out_file)),
                            AssemblerImpl::String => Box::new(PlainTextAssembler::new(&mut out_file)),
                        };
                    assembler.assemble_chunk(thefunc.chunk)?;
                }
            },
            ReplItem::Declaration(decl) => {
                let sym = decl.information().current_symbol_table.clone();
                debug!("sym: {:?}", sym.borrow());
                let type_cker = TypeCk::new(sym);
                let type_check = type_cker.transform_tree(&decl)?;
                let chunk = tbc.transform_tree(&type_check)?;
                debug!("chunk: {:?}", chunk);
            
                {
                    let mut assembler: Box<dyn Assembler<Out = Result<(), AssembleError>>> =
                        match options.assembler {
                            AssemblerImpl::Basic => Box::new(BasicAssembler::new(&mut out_file)),
                            AssemblerImpl::String => Box::new(PlainTextAssembler::new(&mut out_file)),
                        };
                    assembler.assemble_chunk(chunk)?;
                }
            },
        };

        out_file.flush()?;
    }


    Ok(())
}
