# Claw Code

Claw Code is an open source Claude Code with Rust implementation. It is a local coding-agent CLI that provides an interactive agent shell, one-shot prompts, workspace-aware tools, plugin support, and resumeable sessions.

![View Count](https://komarev.com/ghpvc/?username=soongenwong&label=Total%20views&color=ffa500&style=for-the-badge)

<p align="center">
  <a href="https://star-history.com/#soongenwong/claudecode&Date">
    <picture>
      <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/svg?repos=soongenwong/claaudecode&type=Date&theme=dark" />
      <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/svg?repos=soongenwong/claudecode&type=Date" />
      <img alt="Star History Chart" src="https://api.star-history.com/svg?repos=soongenwong/claudecode&type=Date" width="600" />
    </picture>
  </a>
</p>

## Start here

1. Install Rust stable and Cargo.
2. Set up credentials for the model provider you want to use.
3. Run the CLI from the `rust/` workspace.

## First run

From the repository root:

```bash
cd rust
```

Build the CLI:

```bash
cargo build --release -p claw-cli
```

Start the interactive shell:

```bash
cargo run --bin claw -- --help
cargo run --bin claw --
```

Run a single prompt:

```bash
cargo run --bin claw -- prompt "summarize this workspace"
```

Install it locally if you want the binary on your `PATH`:

```bash
cargo install --path crates/claw-cli --locked
```

## Authentication

Use whichever provider you have access to.

Anthropic-compatible models:

```bash
export ANTHROPIC_API_KEY="..."
export ANTHROPIC_BASE_URL="https://api.anthropic.com"
```

Grok models:

```bash
export XAI_API_KEY="..."
export XAI_BASE_URL="https://api.x.ai"
```

You can also authenticate through the CLI:

```bash
cargo run --bin claw -- login
```

## Common commands

```bash
cargo run --bin claw -- prompt "review the latest changes"
cargo run --bin claw -- init
cargo run --bin claw -- logout
cargo run --bin claw -- --resume session.json /status
```

Run `claw --help` for the full command list, including agents, skills, system-prompt output, and slash-command flows.

## Repository layout

```text
.
├── rust/            # Active Rust workspace and CLI/runtime implementation
├── src/             # Python porting workspace and support code
├── tests/           # Python verification
├── CLAW.md          # Repo-specific working notes
└── README.md        # This guide
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

- This repo is an open source Claude Code-style Rust implementation, not the original Claude Code source.
- `CLAW.md` contains the workflow guidance for contributors working in this tree.
