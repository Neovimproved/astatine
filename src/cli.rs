use clap::Parser;

#[derive(Parser)]
pub struct Cli {
    #[arg(long)]
    pub tokens: bool,

    #[arg(long)]
    pub ast: bool,

    #[arg(long)]
    pub asm: bool,

    pub file_name: String,
}
