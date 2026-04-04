# PARITY GAP ANALYSIS

Scope: read-only comparison between the original TypeScript source tree and the Rust port under `rust/crates/`.

Method: compared feature surfaces, registries, entrypoints, and runtime plumbing only. No TypeScript source was copied.

## Executive summary

The Rust port has a good foundation for:
- provider-routing and OAuth basics
- local conversation/session state
- a core tool loop
- MCP stdio/bootstrap support
- CLUES.md discovery
- a small but usable built-in tool set

It is **not feature-parity** with the TypeScript CLI.

Largest gaps:
- **plugins** are effectively absent in Rust
- **hooks** are parsed but not executed in Rust
- **CLI breadth** is much narrower in Rust
- **skills** are local-file only in Rust, without the TS registry/bundled pipeline
- **assistant orchestration** lacks TS hook-aware orchestration and remote/structured transports
- **services** beyond core API/OAuth/MCP are mostly missing in Rust

---

The detailed notes in this file are intentionally internal and operational. Keep using it as a porting ledger rather than user-facing product documentation.
