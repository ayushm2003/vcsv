use std::{
    env, fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use hex::encode as hex_encode;
use vcsv_lib::hash;
use vcsv_script::{inclusion_proof, verify_inclusion, InclusionProofString};

fn tmpdir() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = env::temp_dir().join(format!("vcsv_test_{nonce}"));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn fold_to_root(leaf: [u8; 32], siblings: &[[u8; 32]], mut idx: usize) -> [u8; 32] {
    let mut cur = leaf;
    for sib in siblings {
        let mut buf = [0u8; 64];
        if idx % 2 == 0 {
            buf[..32].copy_from_slice(&cur);
            buf[32..].copy_from_slice(sib);
        } else {
            buf[..32].copy_from_slice(sib);
            buf[32..].copy_from_slice(&cur);
        }
        cur = hash(&buf);
        idx /= 2;
    }
    cur
}

#[test]
fn inclusion_round_trip_ok() {
    let dir = tmpdir();
    let path = dir.join("data.csv");
    fs::write(&path, "id,price,qty\n1,120,3\n2,80,1\n3,200,5\n").unwrap();

    let row: usize = 1;
    let proof_bytes = inclusion_proof(path.clone(), row);
    assert!(!proof_bytes.siblings.is_empty());

    let root = fold_to_root(proof_bytes.leaf, &proof_bytes.siblings, row);

    let proof_hex = InclusionProofString {
        leaf: format!("0x{}", hex_encode(proof_bytes.leaf)),
        siblings: proof_bytes
            .siblings
            .iter()
            .map(|h| format!("0x{}", hex_encode(h)))
            .collect(),
    };

    assert!(verify_inclusion(&root, proof_hex, row));
}

#[test]
fn inclusion_fails_when_sibling_tampered() {
    let dir = tmpdir();
    let path = dir.join("data.csv");
    fs::write(&path, "id,price,qty\n1,120,3\n2,80,1\n3,200,5\n4,150,2\n").unwrap();

    let row: usize = 2;
    let mut proof_bytes = inclusion_proof(path.clone(), row);
    let correct_root = fold_to_root(proof_bytes.leaf, &proof_bytes.siblings, row);

    proof_bytes.siblings[0][0] ^= 0x01;

    let bad_hex = InclusionProofString {
        leaf: format!("0x{}", hex_encode(proof_bytes.leaf)),
        siblings: proof_bytes
            .siblings
            .iter()
            .map(|h| format!("0x{}", hex_encode(h)))
            .collect(),
    };

    assert!(!verify_inclusion(&correct_root, bad_hex, row));
}

#[test]
fn inclusion_handles_odd_leaf_duplication() {
    let dir = tmpdir();
    let path = dir.join("data.csv");
    fs::write(&path, "id,price,qty\n1,10,1\n2,20,2\n3,30,3\n").unwrap();

    let row: usize = 2;
    let proof_bytes = inclusion_proof(path.clone(), row);
    let root = fold_to_root(proof_bytes.leaf, &proof_bytes.siblings, row);

    let proof_hex = InclusionProofString {
        leaf: format!("0x{}", hex_encode(proof_bytes.leaf)),
        siblings: proof_bytes
            .siblings
            .iter()
            .map(|h| format!("0x{}", hex_encode(h)))
            .collect(),
    };

    assert!(verify_inclusion(&root, proof_hex, row));
}
