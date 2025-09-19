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

#[derive(ValueEnum, Clone, Debug, Serialize, Deserialize)]
pub enum Backend {
    Cpu,
    Network,
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
        bytes32 fileRoot;
        uint8 op;
        bytes32 colHash;
        uint64 n_rows;
        int128 result;
        uint16 decimal_points;
    }
}

pub fn sum_col(csv: &Csv) -> (u64, i128, u16) {
    let mut n_rows: u64 = 0;
    let mut sum: i128 = 0;

    for line in &csv.lines {
        if line.is_empty() {
            continue;
        }
        n_rows += 1;
        let val_str = &line[csv.idx];
        let v: i128 = parse_i128(val_str);
        sum = sum.checked_add(v).expect("sum overflow");
    }

    (n_rows, sum, 0)
}

pub fn mean_col(csv: &Csv) -> (u64, i128, u16) {
    let (n_rows, sum, _) = sum_col(csv);

    if n_rows == 0 {
        return (0, 0, 0); // TODO: handle error better
    }

    let decimal: u16 = 3;
    let multiplier = 10_i128.pow(decimal as u32);
    let mean_float = (sum as f64 * multiplier as f64) / (n_rows as f64);
    let mean_scaled = mean_float.round() as i128;

    (n_rows, mean_scaled, decimal)
}

pub fn hash(s: &[u8]) -> [u8; 32] {
    let mut keccak = Keccak::v256();
    let mut hash = [0u8; 32];
    keccak.update(s);
    keccak.finalize(&mut hash);

    hash
}

pub fn parse_csv(csv: Vec<u8>, col: Option<&str>) -> Csv {
    let s = core::str::from_utf8(&csv).expect("csv not utf8");
    let mut lines_iter = s.split("\n");

    let headers = lines_iter.next().expect("empty csv").to_string();
    let cols: Vec<String> = headers
        .split(",")
        .map(trim_ascii)
        .map(|s| s.to_string())
        .collect();

    let idx: usize= match col {
		Some(c) => {
			cols.iter().
				position(|name| name == c)
				.expect("column not found")
		},
		None => 0,
	};

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

pub fn merkelize(csv: &Csv) -> [u8;32] {
	let mut hashes: Vec<[u8; 32]> = csv.lines.iter().map(|line| {
									let row = line.join(",");
									hash(row.as_bytes())
								}).collect();
	
	while hashes.len() > 1 {
		let pairs = hashes.chunks(2);
		hashes = pairs.map(|pair| {
			let left = pair[0];
			let right = if pair.len() == 2 { pair[1] } else { pair[0] };

			let mut buf = [0u8; 64];
			buf[..32].copy_from_slice(&left);
			buf[32..].copy_from_slice(&right);

			hash(&buf)
		}).collect();
	}

	hashes[0]
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

pub fn op_to_u8(op: Op) -> u8 {
    match op {
        Op::Sum => 0,
        Op::Mean => 1,
        Op::Median => 2,
        Op::Hash => 3,
    }
}

fn parse_i128(s: &str) -> i128 {
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
