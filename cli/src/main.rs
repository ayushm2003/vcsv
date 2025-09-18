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
        Command::Execute(ex_args) => execute(ex_args.file, ex_args.ops, ex_args.col),
        Command::Prove(pr_args) => proof(
            pr_args.file,
            pr_args.ops,
            pr_args.col,
            pr_args.out,
            pr_args.backend,
            pr_args.pkey,
        ),
        Command::Verify(vr_args) => verify(vr_args.proof),
    }
}
