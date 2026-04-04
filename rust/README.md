# Clues Code

Clues Code is a local coding-agent CLI implemented in safe Rust. It is built as a clean-room implementation focused on strong local agent workflows, workspace context, and provider-flexible model routing.

The Rust workspace is the current main product surface. The preferred binary name is `clues`, and the CLI provides interactive sessions, one-shot prompts, workspace-aware tools, local agent workflows, and plugin-capable operation from a single workspace.

An optional `../gateway/` service can sit in front of multiple providers behind a generic OpenAI-compatible endpoint, but the primary setup is still direct provider access from the CLI.

## Current status

- **Version:** `0.1.0`
- **Release stage:** initial public release, source-build distribution
- **Primary implementation:** Rust workspace in this repository
- **Platform focus:** macOS and Linux developer workstations

## Install, build, and run

### Prerequisites

- Rust stable toolchain
- Cargo
- Provider credentials for the model you want to use

### Authentication

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

Grok models:

```bash
export XAI_API_KEY="..."
# Optional when using a compatible endpoint
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

OAuth login is also available when you configure a custom OAuth deployment in `.clues.json`:

```bash
cargo run --bin clues -- login
```

### Build from source

```bash
cargo build --release --bin clues
```

### Run

From the workspace:

```bash
cargo run --bin clues -- --help
cargo run --bin clues --
cargo run --bin clues -- prompt "summarize this workspace"
cargo run --bin clues -- --model deepseek-v3 "review the latest changes"
```

From the release build:

```bash
./target/release/clues
./target/release/clues prompt "explain crates/runtime"
```

## Supported capabilities

- Interactive REPL and one-shot prompt execution
- Saved-session inspection and resume flows
- Built-in workspace tools for shell, file read/write/edit, search, web fetch/search, todos, and notebook updates
- Slash commands for status, compaction, config inspection, diff, export, session management, and version reporting
- Local agent and skill discovery with `clues agents` and `clues skills`
- Plugin discovery and management through the CLI and slash-command surfaces
- OAuth login/logout plus model/provider selection from the command line
- Workspace-aware instruction/config loading (`CLUES.md`, compatibility instruction files, config files, permissions, plugin settings)

## Current limitations

- Public distribution is **source-build only** today; this workspace is not set up for crates.io publishing
- GitHub CI verifies `cargo check`, `cargo test`, and release builds, but automated release packaging is not yet present
- Current CI targets Ubuntu and macOS; Windows release readiness is still to be established
- Some live-provider integration coverage is opt-in because it requires external credentials and network access
- The optional OAuth login flow is only relevant for custom deployments; GitHub Models, OpenRouter, Hugging Face, xAI, and other OpenAI-compatible providers are configured with environment variables or config files
- On April 3, 2026, the tested GitHub free models worked in a minimal workspace but still rejected this repo's full startup payload once the request crossed their current ~8k request-size cap
- `clues/...` model names are optional gateway profiles over `OPENAI_BASE_URL`, not a separate hosted Clues provider
- The command surface may continue to evolve during the `0.x` series

## Implementation

The Rust workspace is the active product implementation. It currently includes these crates:

- `clues-cli` - user-facing binary package
- `api` - provider clients and streaming
- `runtime` - sessions, config, permissions, prompts, and runtime loop
- `tools` - built-in tool implementations
- `commands` - slash-command registry and handlers
- `plugins` - plugin discovery, registry, and lifecycle support
- `lsp` - language-server protocol support types and process helpers
- `server` and `compat-harness` - supporting services and compatibility tooling

## Roadmap

- Publish packaged release artifacts for public installs
- Add a repeatable release workflow and longer-lived changelog discipline
- Expand platform verification beyond the current CI matrix
- Add more task-focused examples and operator documentation
- Continue tightening feature coverage and UX polish across the Rust implementation

## Release notes

- Draft 0.1.0 release notes: [`docs/releases/0.1.0.md`](docs/releases/0.1.0.md)

## License

See the repository root for licensing details.
