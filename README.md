# Clues Code

Clues Code is a local-first coding-agent CLI implemented in Rust. It keeps the agent on your machine for filesystem and shell access, while staying provider-flexible through GitHub Models, OpenRouter, Hugging Face router, xAI, generic OpenAI-compatible endpoints, and an optional Railway-friendly gateway.

This repository also includes an optional `gateway/` service for Railway or local deployment. The intended architecture is a local CLI plus an optional provider gateway, not a hosted replacement for the local agent loop.

## Start here

1. Install Rust stable and Cargo.
2. Configure credentials for at least one provider.
3. Run the CLI from the `rust/` workspace.

## Build and run

From the repository root:

```bash
cd rust
cargo build --release --bin clues
cargo run --bin clues -- --help
cargo run --bin clues --
cargo run --bin clues -- prompt "summarize this workspace"
```

On Windows you can also use the local launchers in `Documents`:

```powershell
C:\Users\Amanda\Documents\Install-Clues-Code.cmd
C:\Users\Amanda\Documents\Start-Clues-Code.cmd
```

## Authentication

GitHub Models:

```bash
export GITHUB_TOKEN="..."
# Optional; defaults to https://models.github.ai/inference
export GITHUB_MODELS_BASE_URL="https://models.github.ai/inference"

cargo run --bin clues -- --model deepseek-v3
```

OpenRouter:

```bash
export OPENROUTER_API_KEY="..."
# Optional; defaults to https://openrouter.ai/api/v1
export OPENROUTER_BASE_URL="https://openrouter.ai/api/v1"

cargo run --bin clues -- --model openrouter/auto
```

Hugging Face router:

```bash
export HF_TOKEN="..."
# Optional; defaults to https://router.huggingface.co/v1
export HUGGINGFACE_BASE_URL="https://router.huggingface.co/v1"

cargo run --bin clues -- --model Qwen/Qwen3-Coder-Next:fastest
```

xAI models:

```bash
export XAI_API_KEY="..."
export XAI_BASE_URL="https://api.x.ai"
```

Generic OpenAI-compatible gateways:

```bash
export OPENAI_API_KEY="..."
export OPENAI_BASE_URL="https://your-gateway.example/v1"

cargo run --bin clues -- --model your-model-name
```

Optional Clues gateway profiles over that same OpenAI-compatible endpoint:

```bash
export OPENAI_API_KEY="..."
export OPENAI_BASE_URL="https://your-gateway.example/v1"

cargo run --bin clues -- --model clues/coder-fast
```

You can also authenticate through the CLI if you configured a custom OAuth deployment in `.clues.json`:

```bash
cargo run --bin clues -- login
```

## Common commands

```bash
cargo run --bin clues -- prompt "review the latest changes"
cargo run --bin clues -- init
cargo run --bin clues -- logout
cargo run --bin clues -- --resume session.json /status
```

Run `clues --help` for the full command list, including agents, skills, system-prompt output, and slash-command flows.

Provider notes:

- The CLI respects a configured `model` from settings files, so provider defaults do not require passing `--model` on every launch.
- GitHub Models aliases include `deepseek-v3` and `deepseek-v3-0324`, which resolve to `deepseek/DeepSeek-V3-0324`.
- OpenRouter uses `OPENROUTER_API_KEY` and defaults to `https://openrouter.ai/api/v1`.
- Hugging Face router uses `HF_TOKEN` or `HUGGINGFACE_API_KEY` and defaults to `https://router.huggingface.co/v1`.
- GitHub Models uses `GITHUB_TOKEN` and defaults to `https://models.github.ai/inference`.
- On April 3, 2026, the tested GitHub free models worked in a minimal workspace but still rejected this repo's full startup payload once the request crossed their current ~8k request-size cap.
- Generic OpenAI-compatible gateways use `OPENAI_API_KEY` and `OPENAI_BASE_URL`.
- `clues/...` model names are treated as optional gateway profiles and only route through `OPENAI_BASE_URL`; they are not a standalone Clues-hosted API.

## Repository layout

```text
.
|-- rust/      # Active Rust workspace and CLI/runtime implementation
|-- src/       # Python porting workspace and support code
|-- tests/     # Python verification
|-- CLUES.md   # Repo-specific working notes
`-- README.md  # This guide
```

## Development

Use the Rust workspace for local verification:

```bash
cd rust
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

If you change the Python workspace in `src/`, keep the matching tests in `tests/` updated too.

## Notes

- This repo is a clean-room Rust coding-agent implementation; it does not reuse proprietary upstream source.
- Project-local compatibility files remain supported on this Windows setup while the user-facing workflow centers on `CLUES.md`.
