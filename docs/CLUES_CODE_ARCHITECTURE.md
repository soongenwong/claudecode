# Clues Code Architecture

## Goal

`Clues Code` is a local coding-agent CLI with a provider-flexible model layer.

The target shape is:

- local CLI for filesystem access, shell tools, sessions, and workspace context
- provider-agnostic model routing
- user-owned API keys
- low-latency defaults
- graceful fallback across free and low-cost providers

## Runtime split

### Local CLI

The Rust CLI remains local because it needs:

- direct file reads and writes
- local shell access
- git awareness
- session persistence
- low-friction desktop launch

### Gateway

The optional Railway-hosted `gateway/` service exists to:

- normalize provider endpoints
- centralize fallback logic
- expose user-defined profile names if you choose to run one
- reduce provider-specific setup in the CLI

The gateway is OpenAI-compatible on purpose so the CLI can use its existing generic OpenAI transport.

## Provider strategy

### Direct local providers

The local CLI currently supports:

- GitHub Models
- OpenRouter
- Hugging Face router
- generic OpenAI-compatible backends
- xAI
- optional custom OAuth deployment paths

### Gateway profiles

If the gateway is enabled, it can expose custom profile names such as:

- `coder-fast`
- `coder-balanced`
- `reasoner`
- `github-fast`

These profiles prefer different providers and fail over when one is throttled.

## Config direction

Current Clues Code paths should be:

- `CLUES.md`
- `.clues/`
- `.clues.json`
- `%LOCALAPPDATA%\\CluesCode` on Windows for user-level state

## Near-term roadmap

1. Finish user-facing rebrand cleanup across docs and scaffolding.
2. Validate the optional Railway gateway with a real deployment URL.
3. Add `/v1/models` backed by live provider availability instead of a static tested list.
4. Add provider health scoring and cooldown windows in the gateway.
5. Add usage budgeting and per-provider max-token policies.
6. Add a startup selector or config-driven default profile for desktop use.
7. Move the extracted tree into the new private GitHub repo and start normal PR-based iteration.

## Non-goals for v1

- replacing the local CLI with a hosted app
- adding a database before routing and reliability are stable
- recreating every feature from every commercial coding assistant immediately

The correct first milestone is a reliable private coding CLI with provider fallback and sane local UX.
