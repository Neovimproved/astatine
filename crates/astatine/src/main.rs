use std::{
    error::Error,
    fs::{self, File},
    io::{self, Write},
    path::{Path, PathBuf},
};

use clap::Parser as ClapParser;

mod cli;
mod lexer;
mod parser;
mod span;
mod syntax;

use cli::Cli;
use miette::{Context, IntoDiagnostic, bail};
use soft_canonicalize::soft_canonicalize;

use crate::{lexer::Lexer, parser::Parser, span::to_coordinates};

struct FileWriter {
    relative_path: String,
    input_path: PathBuf,
}

impl FileWriter {
    fn write<E: Error + Send + Sync + 'static>(
        &self,
        path: impl AsRef<Path>,
        f: impl FnOnce(File) -> Result<(), E>,
    ) -> miette::Result<()> {
        let resolved_path = soft_canonicalize(&path)
            .into_diagnostic()
            .wrap_err_with(|| format!("Failed to canonicalize path {}", path.as_ref().display()))?;

        if resolved_path == self.input_path {
            bail!(
                "Unable to continue since the contents of `{}` would otherwise be overriden",
                self.relative_path
            );
        }

        let file = File::create(path)
            .into_diagnostic()
            .wrap_err("Failed to create file")?;

        f(file).into_diagnostic()
    }

    #[allow(dead_code)]
    fn try_on_relative<T, E>(
        &self,
        relative_path: &Path,
        f: impl FnOnce() -> Result<T, E>,
    ) -> miette::Result<T>
    where
        E: Error + Send + Sync + 'static,
    {
        let resolved_path = soft_canonicalize(relative_path)
            .into_diagnostic()
            .wrap_err_with(|| format!("Failed to canonicalize {}", relative_path.display()))?;

        if self.input_path == resolved_path {
            bail!(
                "Unable to continue since the contents of `{}` would otherwise be overriden",
                self.relative_path
            );
        }

        f().into_diagnostic()
    }
}

impl TryFrom<String> for FileWriter {
    type Error = io::Error;

    fn try_from(value: String) -> io::Result<Self> {
        let input_path = fs::canonicalize(&value)?;

        Ok(Self {
            relative_path: value,
            input_path,
        })
    }
}

fn dedent(s: &str, mut f: impl FnMut(usize) -> usize) -> String {
    let mut buf = String::new();

    for line in s.lines() {
        let spaces = line.chars().take_while(|ch| *ch == ' ').count();
        buf.push_str(&line[spaces - f(spaces)..]);
        buf.push('\n');
    }

    if !s.ends_with('\n') {
        buf.pop();
    }

    buf
}

fn main() -> miette::Result<()> {
    let args = Cli::parse();

    let file_name = args.file_name;
    let if_path = Path::new(&file_name);

    let file_writer = FileWriter::try_from(file_name.clone()).into_diagnostic()?;

    let input_file_string = fs::read_to_string(&file_name)
        .into_diagnostic()
        .wrap_err("Failed to read input file")?;

    let if_chars: Vec<char> = input_file_string.chars().collect();

    let mut lexer = Lexer::new(if_chars);

    let tokens = match lexer.tokenize() {
        Ok(tokens) => tokens,
        Err(err) => {
            let (line, column) = to_coordinates(&input_file_string, lexer.pos());
            bail!("Failed to tokenize with {err}\nat line {line}, column {column}",);
        }
    };

    let dedent_by_2 = |n| n / 2;

    if cfg!(debug_assertions) || args.tokens {
        file_writer.write(if_path.with_extension("tok"), |mut file| {
            write!(file, "{}", dedent(&format!("{tokens:#?}"), dedent_by_2))
        })?;
    }

    let mut parser = Parser::new(tokens);

    let ast = match parser.parse() {
        Ok(ast) => ast,
        Err(err) => {
            let pos = match parser.peek() {
                Some(token) => token.pos(),
                None => {
                    bail!("Failed to parse\n{err}at EOF");
                }
            };

            let (line, column) = to_coordinates(&input_file_string, pos);

            bail!("Failed to parse\n{err}\nat line {line}, column {column}");
        }
    };

    if cfg!(debug_assertions) || args.ast {
        file_writer.write(if_path.with_extension("ast"), |mut file| {
            write!(file, "{}", dedent(&format!("{ast:#?}"), dedent_by_2))
        })?;
    }

    if cfg!(debug_assertions) || args.asm {
        // write asm
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn dedent_test() {
        let s = "    I love tacos\n    me too\n       okay that's cool";

        assert_eq!(
            dedent(s, |n| n / 2),
            "  I love tacos\n  me too\n   okay that's cool"
        );
    }
}
