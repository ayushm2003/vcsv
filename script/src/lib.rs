//! VCSV Script Library
//!
//! This library provides functions for executing and proving the vcsv program

use alloy_sol_types::SolType;
use sp1_sdk::{include_elf, ProverClient, SP1Stdin};
use std::fs;
use std::path::PathBuf;
use vcsv_lib::{Input, Op, PublicValues};

/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
pub const VCSV_ELF: &[u8] = include_elf!("vcsv-program");

/// Execute the CSV program without generating a proof
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
        fileHash,
        op,
        colHash,
        n_rows,
        result,
    } = decoded;

    println!("fileHash: {:?}", fileHash);
    println!("op: {:?}", op);
    println!("colHash: {:?}", colHash);
    println!("n_rows: {:?}", n_rows);
    println!("result: {:?}", result);
}

/// Generate a proof for the CSV program
pub fn proof(file: PathBuf, op: Op, col: String, _out: PathBuf) {
    let client = ProverClient::from_env();
    let mut stdin = SP1Stdin::new();

    let csv_bytes: Vec<u8> = fs::read(file).unwrap();

    let input = Input {
        csv: csv_bytes,
        col: col,
        op: op,
    };
    stdin.write(&input);

    let (pk, vk) = client.setup(VCSV_ELF);

    // Generate the proof
    let proof = client
        .prove(&pk, &stdin)
        .run()
        .expect("failed to generate proof");

    println!("Successfully generated proof!");
	// println!("proof: {:?}", proof);

    client.verify(&proof, &vk).expect("failed to verify proof");
    println!("Successfully verified proof!");
}
