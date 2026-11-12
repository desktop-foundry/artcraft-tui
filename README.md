# artcraft-tui

A production-quality, cross-platform CLI/TUI for discovering, installing, diagnosing,
updating, repairing, and reconciling installations of the ArtCraft creative application suite.

## Build

```sh
cargo build --release
```

Binary: `target/release/artcraft`.

## Usage

```sh
artcraft detect
artcraft catalog
artcraft discover
artcraft diagnose
artcraft plan
artcraft setup
artcraft tui   # or just `artcraft`
```

Machine-readable output: add `--json` to any command.

## Status

Early development. See `.development/` for engineering notes (ignored by git).
EOF
echo done