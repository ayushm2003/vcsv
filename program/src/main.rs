//! A simple program that takes a number `n` as input, and writes the `n-1`th and `n`th fibonacci
//! number as an output.

// These two lines are necessary for the program to properly compile.
//
// Under the hood, we wrap your main function with some extra code so that it behaves properly
// inside the zkVM.
#![no_main]
sp1_zkvm::entrypoint!(main);

use alloy_sol_types::SolType;
use vcsv_lib::{hash, mean_col, op_to_u8, parse_csv, sum_col, Input, Op, PublicValues};

pub fn main() {
    // Read an input to the program.
    //
    // Behind the scenes, this compiles down to a custom system call which handles reading inputs
    // from the prover.
    let Input { csv, col, op } = sp1_zkvm::io::read::<Input>();

    let file_hash = hash(&csv);

    let csv_cont = parse_csv(csv, &col);

    let (n_rows, result, decimal_points) = match op {
        Op::Sum => sum_col(&csv_cont),
        Op::Mean => mean_col(&csv_cont),
        _ => panic!("op not implemented yet"),
    };

    let col_bytes = col.as_bytes();
    let col_hash = hash(col_bytes);

    let public = PublicValues {
        fileHash: file_hash.into(),
        op: op_to_u8(op),
        colHash: col_hash.into(),
        n_rows,
        result,
        decimal_points,
    };

    // Encode the public values of the program.
    let bytes = PublicValues::abi_encode(&public);

    // Commit to the public values of the program. The final proof will have a commitment to all the
    // bytes that were committed to.
    sp1_zkvm::io::commit_slice(&bytes);
}
