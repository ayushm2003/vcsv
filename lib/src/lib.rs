use alloy_sol_types::sol;
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use tiny_keccak::{Hasher, Keccak};

#[derive(ValueEnum, Clone, Debug, Serialize, Deserialize)]
pub enum Op {
    Sum,
    Mean,
    Median,
    Hash,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub csv: Vec<u8>,
    pub col: String,
    pub op: Op,
}

pub struct Csv {
    pub lines: Vec<Vec<String>>,
    pub headers: String,
    pub cols: Vec<String>,
    pub idx: usize,
}

sol! {
    struct PublicValues {
        bytes32 fileHash;
        uint8 op;
        bytes32 colHash;
        uint64 n_rows;
        int128 result;
    }
}

pub fn hash(s: &[u8]) -> [u8; 32] {
    let mut keccak = Keccak::v256();
    let mut hash = [0u8; 32];
    keccak.update(s);
    keccak.finalize(&mut hash);

    hash
}

pub fn parse_csv(csv: Vec<u8>, col: &str) -> Csv {
    let s = core::str::from_utf8(&csv).expect("csv not utf8");
    let mut lines_iter = s.split("\n");

    let headers = lines_iter.next().expect("empty csv").to_string();
    let cols: Vec<String> = headers
        .split(",")
        .map(trim_ascii)
        .map(|s| s.to_string())
        .collect();
    let mut target_idx = None;

    for (i, name) in cols.iter().enumerate() {
        if name == col {
            target_idx = Some(i);
            break;
        }
    }

    let idx = target_idx.expect("col not found");

    // Parse all remaining lines into Vec<Vec<String>>
    let mut lines = Vec::new();
    for line in lines_iter {
        if line.is_empty() {
            continue;
        }
        let row: Vec<String> = line
            .split(",")
            .map(trim_ascii)
            .map(|s| s.to_string())
            .collect();
        lines.push(row);
    }

    Csv {
        lines,
        headers,
        cols,
        idx,
    }
}

pub fn trim_ascii(s: &str) -> &str {
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
