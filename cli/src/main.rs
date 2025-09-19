use clap::{Args, Parser, Subcommand};
use hex::encode;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use vcsv_lib::{Backend, Op};
use vcsv_script::{execute, inclusion_proof, proof, verify};

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
    InclusionProof(InclusionProofArgs),
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

#[derive(Args, Debug)]
pub struct InclusionProofArgs {
    #[arg(long)]
    file: PathBuf,
    #[arg(long)]
    row: u64,
    #[arg(long)]
    pub out: Option<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InclusionProof {
    pub leaf: [u8; 32],
    pub siblings: Vec<[u8; 32]>,
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
        Command::InclusionProof(args) => {
            let proof = inclusion_proof(args.file, args.row as usize); // returns MerkleProof { leaf, siblings }

            #[derive(Serialize)]
            struct ProofOut {
                leaf: String,
                siblings: Vec<String>,
            }

            let out = ProofOut {
                leaf: format!("0x{}", hex::encode(proof.leaf)),
                siblings: proof
                    .siblings
                    .iter()
                    .map(|h| format!("0x{}", hex::encode(h)))
                    .collect(),
            };

            let json = serde_json::to_string_pretty(&out).unwrap();

            match args.out {
                Some(path) => {
                    fs::write(path, json).expect("couldn't write proof to file");
                }
                None => println!("{json}"),
            }
        }
    }
}
