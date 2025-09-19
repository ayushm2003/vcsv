//! VCSV Script Library
//!
//! This library provides functions for executing and proving the vcsv program

use alloy_sol_types::SolType;
use sp1_sdk::{include_elf, ProverClient, SP1Stdin};
use std::{env::set_var, fs, path::PathBuf};
use vcsv_lib::{hash, parse_csv, Backend, Input, Op, PublicValues};

/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
pub const VCSV_ELF: &[u8] = include_elf!("vcsv-program");

pub fn execute(file: PathBuf, op: Op, col: String) {
    let client = ProverClient::from_env();
    let mut stdin = SP1Stdin::new();

    let csv_bytes: Vec<u8> = fs::read(file).unwrap();

    let input = Input {
        csv: csv_bytes,
        col: col,
        op: op,
    };
    stdin.write(&input);

    let (output, _report) = client.execute(VCSV_ELF, &stdin).run().unwrap();
    println!("Program executed successfully.");

    let decoded = PublicValues::abi_decode(output.as_slice()).unwrap();
    let PublicValues {
        fileRoot,
        op,
        colHash,
        n_rows,
        result,
        decimal_points,
    } = decoded;

    println!("fileRoot: {:?}", fileRoot);
    println!("op: {:?}", op);
    println!("colHash: {:?}", colHash);
    println!("n_rows: {:?}", n_rows);
    println!(
        "result: {:?}",
        result as f64 / 10_f64.powf(decimal_points as f64)
    );
    println!("decimal points: {:?}", decimal_points);
}

pub fn proof(
    file: PathBuf,
    op: Op,
    col: String,
    out: PathBuf,
    backend: Backend,
    pkey: Option<String>,
) {
    match backend {
        Backend::Cpu => set_var("SP1_PROVER", "cpu"),
        Backend::Network => {
            set_var("SP1_PROVER", "network");
            set_var("NETWORK_PRIVATE_KEY", pkey.unwrap());
        }
    }
    let client = ProverClient::from_env();
    let mut stdin = SP1Stdin::new();

    let csv_bytes: Vec<u8> = fs::read(file).unwrap();

    let input = Input {
        csv: csv_bytes,
        col: col,
        op: op,
    };
    stdin.write(&input);

    let (pk, _) = client.setup(VCSV_ELF);

    let proof = client
        .prove(&pk, &stdin)
        .compressed()
        .run()
        .expect("failed to generate proof");

    println!("Successfully generated proof!");

    let proof = serde_json::to_vec_pretty(&proof).unwrap();
    let _ = fs::write(out, proof).expect("couldn't write to file");
}

pub fn verify(file: PathBuf) {
    let client = ProverClient::from_env();
    let (_, vk) = client.setup(VCSV_ELF);

    let proof = fs::read(file).unwrap();
    let proof = serde_json::from_slice(&proof).unwrap();

    client.verify(&proof, &vk).expect("failed to verify proof");
    println!("Successfully verified proof!");
}

fn merkle_path(file: PathBuf, row_idx: usize) ->  Vec<[u8; 32]> {
	let csv = parse_csv(fs::read(file).unwrap(), None);

	if row_idx >= csv.lines.len() {
		panic!("row index out of bounds");
	}
	let mut i = row_idx;

	let mut hashes: Vec<[u8; 32]> = Vec::new();

	let mut levels: Vec<[u8; 32]> = csv.lines.iter().map(|line| {
									let row = line.join(",");
									hash(row.as_bytes())
								}).collect();
	
	hashes.push(levels[i]);

	while levels.len() > 1 {
		let sibling_idx = 
			if i % 2 == 0 {
				if i + 1 < levels.len() { i + 1 } else { i }
			} else {
				i - 1
			};
		
		hashes.push(levels[sibling_idx]);

		levels = levels.chunks(2).map(|pair| {
			let left = pair[0];
			let right = if pair.len() == 2 { pair[1] } else { pair[0] };

			let mut buf = [0u8; 64];
			buf[..32].copy_from_slice(&left);
			buf[32..].copy_from_slice(&right);

			hash(&buf)
		}).collect();
		

		i /= 2;
	}

	hashes
}