# Clues Gateway

`Clues Gateway` is a small OpenAI-compatible router for `Clues Code`.

It keeps the coding agent local, but gives the CLI a single endpoint that can:

- route requests across GitHub Models, OpenRouter, and Hugging Face
- expose stable profile names like `clues/coder-fast`
- fail over between providers when one key or free tier is throttled
- run cleanly on Railway without a database

## Why this exists

The local CLI needs filesystem and tool access, so it should stay local.

What benefits from hosting is the provider layer:

- key management
- provider normalization
- fallback order
- lightweight routing rules

This gateway is intentionally stateless. A database is not required for the first version.

## Supported profiles

The first version ships these profile models:

- `clues/coder-fast`
- `clues/coder-balanced`
- `clues/reasoner`
- `clues/github-fast`

It also accepts explicit provider-prefixed models:

- `github::openai/gpt-4.1-mini`
- `openrouter::openrouter/auto`
- `huggingface::Qwen/Qwen3-Coder-Next:fastest`

## Local run

```bash
cd gateway
python -m venv .venv
. .venv/bin/activate
pip install -r requirements.txt
uvicorn app.main:app --reload --port 8080
```

## Railway deploy

1. Point Railway at the `gateway/` folder.
2. Set the environment variables from `.env.example`.
3. Deploy.

Health check:

- `GET /health`

OpenAI-compatible endpoints:

- `GET /v1/models`
- `POST /v1/chat/completions`

## Pointing Clues Code at the gateway

Set these in your user environment:

```powershell
[System.Environment]::SetEnvironmentVariable('OPENAI_BASE_URL', 'https://your-gateway.up.railway.app/v1', 'User')
[System.Environment]::SetEnvironmentVariable('OPENAI_API_KEY', 'your-gateway-token-or-placeholder', 'User')
[System.Environment]::SetEnvironmentVariable('CLUES_MODEL', 'clues/coder-fast', 'User')
```

Then launch:

```powershell
C:\Users\Amanda\Documents\Start-Clues-Code.cmd
```

If you keep direct provider keys set locally as well, the CLI should prefer the gateway for `clues/...` models. The provider-selection patch in the Rust CLI makes that deterministic.
