# Architecture

Rust workspace with library-first crates under `crates/` and a single binary
`apps/artcraft`. Follows Domain-Driven Design and Hexagonal Architecture:
domain has no dependencies on UI, storage, or OS APIs.

See `.development/` for engineering notes and decisions.
