# Example Ledger using Event Sourcing in Rust

## Run

    nix run .#ledger -- tests/transactions.csv > accounts.csv

OR

    cargo run -- transactions.csv > accounts.csv

## Run Checks (clippy, docs, fmt, toml-fmt, audit)

    nix flake check

## Run Tests

    cargo test

## Run Coverage

    cargo tarpaulin
