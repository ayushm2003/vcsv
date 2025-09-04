use alloy_sol_types::sol;
use clap::ValueEnum;
use serde::{Deserialize, Serialize};

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

sol! {
    struct PublicValues {
        bytes32 fileHash;
        uint8 op;
        bytes32 colHash;
        uint64 n_rows;
        int128 result;
    }
}
