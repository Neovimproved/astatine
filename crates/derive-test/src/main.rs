use astatine_derive::EnumTryAs;
use derive_more::{Display, Error};
use miette::IntoDiagnostic;

#[derive(Debug, Display, Error)]
enum Error {
    #[display("Failed to convert to {_0}")]
    Variant(#[error(not(source))] &'static str),
}

#[derive(Debug, EnumTryAs)]
#[error(Error, Variant)]
enum MyEnum {
    Foo { value: String },
    Bar { thing: String },
    Baz(String),
    Qux,
    Quux,
}

fn main() -> miette::Result<()> {
    let variant = MyEnum::Qux.try_as_foo().into_diagnostic()?;

    println!("{variant}");

    Ok(())
}
