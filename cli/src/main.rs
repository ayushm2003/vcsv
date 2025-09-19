use clap::{Args, Parser, Subcommand};
use hex::{decode, encode};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use vcsv_lib::{Backend, Op};
use vcsv_script::{
    execute, inclusion_proof, proof, verify, verify_inclusion, InclusionProofString,
};

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
    VerifyInclusion(VerifyInclusionArgs),
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

#[derive(Args, Debug)]
pub struct VerifyInclusionArgs {
    #[arg(long)]
    pub root: String,
    #[arg(long)]
    pub proof: PathBuf,
    #[arg(long)]
    pub row: usize,
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

            let out = InclusionProofString {
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
        Command::VerifyInclusion(args) => {
            let json = fs::read_to_string(&args.proof).expect("failed to read proof file");
            let inc_proof: InclusionProofString =
                serde_json::from_str(&json).expect("invalid proof JSON");

            let mut root_arr = [0u8; 32];
            let root_bytes = decode(args.root.trim_start_matches("0x")).expect("invalid root hex");
            assert_eq!(root_bytes.len(), 32);
            root_arr.copy_from_slice(&root_bytes);

            let ok = verify_inclusion(&root_arr, inc_proof, args.row);
            println!("{}", if ok { "verified!" } else { "failed :(" });
        }
    }
}
