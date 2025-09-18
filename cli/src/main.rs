use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;
use vcsv_lib::{Backend, Op};
use vcsv_script::{execute, proof, verify};

#[derive(Parser)]
#[command(name = "vcsv", version, about = "Verifiable CSV analytics")]
struct Cli {
    #[command(subcommand)]
    pub cmd: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    Execute(ExecuteArgs),
    Prove(ProveArgs),
    Verify(VerifyArgs),
}

#[derive(Args, Debug)]
pub struct ExecuteArgs {
    #[arg(long, value_enum)]
    pub ops: Op,
    #[arg(long)]
    pub file: PathBuf,
    #[arg(long)]
    pub col: String,
}

#[derive(Args, Debug)]
pub struct ProveArgs {
    #[arg(long, value_enum)]
    pub ops: Op,
    #[arg(long)]
    pub file: PathBuf,
    #[arg(long)]
    pub col: String,
    #[arg(long, default_value = "proof.json")]
    pub out: PathBuf,
    #[arg(long, value_enum, default_value = "cpu")]
    pub backend: Backend,
    #[arg(long, required_if_eq("backend", "network"))]
    pub pkey: Option<String>,
}

#[derive(Args, Debug)]
pub struct VerifyArgs {
    #[arg(long, default_value = "proof.json")]
    pub proof: PathBuf,
}

fn main() {
    let args = Cli::parse();
    println!("cmd={:?}", args.cmd);

    match args.cmd {
        Command::Execute(args) => execute(args.file, args.ops, args.col),
        Command::Prove(args) => proof(
            args.file,
            args.ops,
            args.col,
            args.out,
            args.backend,
            args.pkey,
        ),
        Command::Verify(args) => verify(args.proof),
    }
}
