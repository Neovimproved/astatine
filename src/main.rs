#![allow(unused)]

mod cli;
mod generator;
mod lexer;
mod parser;
mod symbols;
mod syntax;

use std::{fs, path::Path, process::Command};

use clap::Parser as ClapParser;
use cli::Cli;
use color_eyre::eyre::{Context, Result, eyre};

use crate::{generator::Generator, lexer::Lexer, parser::Parser};

fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Cli::parse();

    let file_path = fs::canonicalize(&args.file_name)?;

    let output_file = match args.output {
        Some(out) => out,
        None => Path::new(
            file_path
                .file_name()
                .ok_or_else(|| eyre!("File name could not be resolved"))?,
        )
        .file_stem()
        .ok_or_else(|| eyre!("File stem could not be extracted"))?
        .to_string_lossy()
        .to_string(),
    };

    let canonicalized_output_file = fs::canonicalize(&output_file)?;

    if canonicalized_output_file == file_path {
        return Err(eyre!("Output file cannot be the same as an input file"));
    }

    let contents = fs::read_to_string(&file_path)
        .wrap_err_with(|| format!("Failed to read file `{}`", args.file_name))?;

    let lexer = Lexer::new(&contents);
    let tokens = lexer.tokenize()?;

    if args.tokens {
        let tokens_file = format!("{output_file}.tokens");
        fs::write(tokens_file, format!("{tokens:#?}"))?;
    }

    let ast = Parser::new(&tokens).parse()?;

    if args.ast {
        let ast_file = format!("{output_file}.ast");
        fs::write(ast_file, format!("{ast:#?}"))?;
    }

    let asm = Generator::new(&ast).generate_asm();

    let asm_file = format!("{output_file}.asm");
    fs::write(&asm_file, asm)?;

    let nasm_status = Command::new("nasm")
        .arg("-felf64")
        .arg(&asm_file)
        .status()?;

    if !args.asm {
        fs::remove_file(asm_file)?;
    }

    if !nasm_status.success() {
        return Err(eyre!("Failed to assemble"));
    }

    let object_file = format!("{output_file}.o");

    if !Command::new("ld")
        .arg(&object_file)
        .arg("-o")
        .arg(output_file)
        .status()?
        .success()
    {
        return Err(eyre!("Failed to link object file"));
    };

    if !args.object {
        fs::remove_file(object_file)?;
    }

    Ok(())
}
