# vcsv

vcsv is a Rust command line tool that proves computed analytics over a CSV dataset without revealing the raw data.
It uses Succinct’s SP1 zkVM to generate and verify zero knowledge proofs.

## Features

- Zero-knowledge analytics – prove sum, mean, median on a CSV column without exposing the file.
- Merkle commitment – every CSV row is committed to a Merkle root so you can later prove that a specific row was part of the dataset.
- Dual proving backends – run proofs locally (`--backend cpu`) or on the Succinct Prover Network (`--backend network`).
- Inclusion proofs – generate and verify row level Merkle inclusion proofs.

## Installation

Requires Rust ≥1.73

Clone the repo and install:

```
cargo install --path cli
```

This builds the release binary and puts the vcsv executable in ~/.cargo/bin, so it’s available on your PATH.

(You can also run without installing: `cargo run --release -- <subcommand> [flags]`.)

## Usage

1. Execute analytics

```
vcsv execute --file data.csv --op sum --col price
```

2. Prove and Verify

```
vcsv prove --file data.csv --op mean --col price --out proof.json --backend network --pkey 0x...
```

```
vcsv verify --proof proof.json
```

3. Generate an inclusion proof

```
vcsv inclusion-proof --file data.csv --row 5 --out proof.json
```

4. Verify an inclusion proof

```
vcsv verify-inclusion --root 0x... --proof proof.json --row 5
```

## Examples

[examples](examples) has some csv files to play with.

```
vcsv execute --file examples/tiny.csv --op mean --col price
```

```
vcsv prove --file examples/tiny.csv --op median --col price --out proof.json
```

```
vcsv verify --proof proof.json
```

## Running the tests

The repository includes some tests for the merkle path and inclusion-proof logic.

From the workspace root run:

```
cargo test -p vcsv-script
```

All tests should pass without any additional setup.
