# ClaudeCode (Open Source)

**A high-performance, open-source AI coding agent written in Rust.**

ClaudeCode is a terminal-native CLI agent designed to bring advanced LLM capabilities directly into your development workflow. Built for speed, safety, and efficiency, it provides an interactive agent shell, workspace-aware tools, and persistent session management. It is an independent open-source implementation inspired by Claude Code, not the official Anthropic product.

![View Count](https://komarev.com/ghpvc/?username=soongenwong&label=Total+views&color=ffa500&style=for-the-badge)

## Star History

<a href="https://www.star-history.com/?repos=soongenwong%2Fclaudecode&type=date&legend=bottom-right">
 <picture>
   <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/chart?repos=soongenwong/claudecode&type=date&theme=dark&legend=top-left" />
   <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/chart?repos=soongenwong/claudecode&type=date&legend=top-left" />
   <img alt="ClaudeCode Star History" src="https://api.star-history.com/chart?repos=soongenwong/claudecode&type=date&legend=top-left" />
 </picture>
</a>

## Related Projects

- [Anthropic developer docs](https://platform.claude.com/docs)
- [Anthropic](https://www.anthropic.com/)
- [Anthropic on X](https://x.com/AnthropicAI)

## Key Features

- **Rust-powered:** Built with Rust for memory safety, minimal binary size, and high execution speed.
- **Agentic CLI:** Interactive shell and one-shot prompt support for seamless terminal workflows.
- **Model flexible:** Supports Anthropic-compatible and OpenAI-compatible providers, plus xAI/Grok aliases.
- **Workspace aware:** Context-aware tools designed to understand your local codebase.
- **Session persistence:** Resumeable sessions via JSON state management.
- **Extensible:** Plugin-ready architecture for custom tools and skills.

## Feature Parity Status

This is an MVP implementation. The Rust CLI aims for feature parity with the TypeScript implementation, but currently has the following known gaps:

| Feature | Status | Notes |
|---------|--------|-------|
| **Tools** | ✅ Partial | Core tool registry implemented; MCP tools, remote triggers, and team tools planned |
| **Hooks** | ⚠️ Config only | Parsed from configuration but not executed in runtime (pre/post tool hooks planned) |
| **Plugins** | ❌ Missing | Plugin system architecture designed but not yet implemented |
| **Skills** | ⚠️ Limited | Local file-based skills supported; registry-based discovery planned |
| **MCP Integration** | ✅ Partial | Core MCP client implemented; full workspace diagnostics planned |
| **LSP Integration** | ⚠️ Partial | Basic LSP support; diagnostics integration in progress |

**Current Status:** MVP feature set suitable for basic coding assistance. Advanced customization features (plugins, hooks) are planned for future releases.

For detailed feature gaps, see [PARITY.md](./PARITY.md).

## Getting Started

### Prerequisites

1. [Install Rust](https://www.rust-lang.org/tools/install) stable and Cargo.
2. Set up your preferred API credentials.

### Installation

From the repository root:

```bash
cd rust
cargo build --release -p claw-cli

# Install locally to your PATH for global access
cargo install --path crates/claw-cli --locked
```

### Usage

Start the interactive shell:

```bash
claw
```

Run a single prompt:

```bash
claw prompt "summarize this workspace"
```

Resume a previous session:

```bash
claw --resume session.json /status
```

Run `claw --help` for the full command list, including agents, skills, and system-prompt flows.

## Authentication

Configure your environment variables based on your preferred provider:

### Anthropic

```bash
export ANTHROPIC_API_KEY="..."
export ANTHROPIC_BASE_URL="https://api.anthropic.com"
```

### OpenAI-compatible

```bash
export OPENAI_API_KEY="..."
export OPENAI_BASE_URL="https://api.openai.com/v1"
```

### Grok / xAI

```bash
export XAI_API_KEY="..."
export XAI_BASE_URL="https://api.x.ai"
```

You can also authenticate via the CLI:

```bash
claw login
```

## Frequently Asked Questions

**What is this project?** This is an independent, open-source implementation of a terminal-based coding agent, architecturally inspired by Claude Code.

**Why Rust?** Rust provides the performance, concurrency, and memory safety required for a tool that interacts deeply with local file systems and high-latency LLM APIs.

**Can I use local models?** Yes, if your local inference server exposes an OpenAI-compatible API and you point the relevant base URL and API key at it.

**Is this the official Anthropic Claude Code?** No, this is a community-driven open-source project.

## Development

We welcome contributions. Please refer to CLAW.md for workspace-specific workflow guidance.

```bash
cd rust
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Repository Structure

- `rust/`: Core CLI and runtime implementation.
- `src/`: Python support code and utilities.
- `tests/`: Verification suites for agentic behaviors.
- `CLAW.md`: Internal workflow documentation.

## Notes

- This project is an open-source implementation.
- It is not affiliated with or endorsed by Anthropic.
