//! A simple program that takes a number `n` as input, and writes the `n-1`th and `n`th fibonacci
//! number as an output.

// These two lines are necessary for the program to properly compile.
//
// Under the hood, we wrap your main function with some extra code so that it behaves properly
// inside the zkVM.
#![no_main]
sp1_zkvm::entrypoint!(main);

use alloy_sol_types::SolType;
use tiny_keccak::{Hasher, Keccak};
use vcsv_lib::{Input, Op, PublicValues};

pub fn main() {
    // Read an input to the program.
    //
    // Behind the scenes, this compiles down to a custom system call which handles reading inputs
    // from the prover.
    let Input { csv, col, op } = sp1_zkvm::io::read::<Input>();

    let mut keccak = Keccak::v256();
    keccak.update(&csv);
    let mut file_hash = [0u8; 32];
    keccak.finalize(&mut file_hash);

    // parse csv
    let s = core::str::from_utf8(&csv).expect("csv not utf8");
    let mut lines = s.split("\n");

    let headers = lines.next().expect("empty csv");
    let cols = headers.split(",").map(trim_ascii);
    let mut target_idx = None;

    for (i, name) in cols.enumerate() {
        if name == col {
            target_idx = Some(i);
            break;
        }
    }

    let idx = target_idx.expect("col not found");

    let mut n_rows: u64 = 0;
    let mut sum: i128 = 0;

    for line in lines {
        if line.is_empty() {
            continue;
        }
        n_rows += 1;
        let mut it = line.split(',').map(trim_ascii);
        let val_str = it.nth(idx).expect("missing field");
        let v: i128 = parse_i128(val_str);

        sum = sum.checked_add(v).expect("sum overflow");
    }

    let result = match op {
        Op::Sum => sum,
        _ => panic!("op not implemented yet"),
    };

    let col_bytes = col.as_bytes();
    let mut col_hash = [0u8; 32];
    let mut keccak = Keccak::v256();
    keccak.update(col_bytes);
    keccak.finalize(&mut col_hash);

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
fn trim_ascii(s: &str) -> &str {
    let bytes = s.as_bytes();
    let mut i = 0;
    let mut j = bytes.len();
    while i < j && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    while j > i && bytes[j - 1].is_ascii_whitespace() {
        j -= 1;
    }
    unsafe { core::str::from_utf8_unchecked(&bytes[i..j]) }
}

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
