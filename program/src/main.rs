//! A simple program that takes a number `n` as input, and writes the `n-1`th and `n`th fibonacci
//! number as an output.

// These two lines are necessary for the program to properly compile.
//
// Under the hood, we wrap your main function with some extra code so that it behaves properly
// inside the zkVM.
#![no_main]
sp1_zkvm::entrypoint!(main);

use alloy_sol_types::SolType;
use vcsv_lib::{hash, parse_csv, Csv, Input, Op, PublicValues};

pub fn main() {
    // Read an input to the program.
    //
    // Behind the scenes, this compiles down to a custom system call which handles reading inputs
    // from the prover.
    let Input { csv, col, op } = sp1_zkvm::io::read::<Input>();

    let file_hash = hash(&csv);

    let Csv {
        lines,
        headers,
        cols,
        idx,
    } = parse_csv(csv, &col);

    let mut n_rows: u64 = 0;
    let mut sum: i128 = 0;

    for line in lines {
        if line.is_empty() {
            continue;
        }
        n_rows += 1;
        let val_str = &line[idx];
        let v: i128 = parse_i128(val_str);

        sum = sum.checked_add(v).expect("sum overflow");
    }

    let result = match op {
        Op::Sum => sum,
        _ => panic!("op not implemented yet"),
    };

    let col_bytes = col.as_bytes();
    let mut col_hash = hash(col_bytes);

    let public = PublicValues {
        fileHash: file_hash.into(),
        op: op_to_u8(op),
        colHash: col_hash.into(),
        n_rows,
        result,
    };

    // Encode the public values of the program.
    let bytes = PublicValues::abi_encode(&public);

    // Commit to the public values of the program. The final proof will have a commitment to all the
    // bytes that were committed to.
    sp1_zkvm::io::commit_slice(&bytes);
}

// helpers

fn parse_i128(s: &str) -> i128 {
    // no floats, no underscores; optional leading '-'
    // tiny, deterministic parser to avoid locale/float issues
    let b = s.as_bytes();
    let mut i = 0usize;
    let neg = if !b.is_empty() && b[0] == b'-' {
        i = 1;
        true
    } else {
        false
    };
    let mut acc: i128 = 0;
    while i < b.len() {
        let d = b[i];
        if d < b'0' || d > b'9' {
            panic!("non-digit in integer");
        }
        acc = acc
            .checked_mul(10)
            .and_then(|x| x.checked_add((d - b'0') as i128))
            .expect("int overflow");
        i += 1;
    }
    if neg {
        -acc
    } else {
        acc
    }
}

fn op_to_u8(op: Op) -> u8 {
    match op {
        Op::Sum => 0,
        Op::Mean => 1,
        Op::Median => 2,
        Op::Hash => 3,
    }
}
