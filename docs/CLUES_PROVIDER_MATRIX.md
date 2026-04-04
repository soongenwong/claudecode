# Clues Provider Matrix

This matrix reflects live testing done from the local Windows machine during the Clues Code bootstrap.

## Official direct setup

- GitHub Models
  - Docs: <https://docs.github.com/en/rest/models/inference>
  - Base URL: `https://models.github.ai/inference`
  - Auth: `GITHUB_TOKEN`
  - Notes: model IDs use the `{publisher}/{model}` form. The direct REST examples also send `Accept: application/vnd.github+json` and `X-GitHub-Api-Version`.

- OpenRouter
  - Docs: <https://openrouter.ai/docs/quickstart>
  - Base URL: `https://openrouter.ai/api/v1`
  - Auth: `OPENROUTER_API_KEY`
  - Notes: optional attribution headers are `HTTP-Referer` and `X-OpenRouter-Title`.

- Hugging Face router
  - Docs: <https://huggingface.co/docs/inference-providers/index>
  - Base URL: `https://router.huggingface.co/v1`
  - Auth: `HF_TOKEN` or `HUGGINGFACE_API_KEY`
  - Notes: `:fastest`, `:cheapest`, and `:preferred` suffixes control provider selection policy on the router.

## Optional gateway setup

- Gateway profiles like `clues/coder-fast` are only valid when `OPENAI_BASE_URL` points at the optional `gateway/` service or another OpenAI-compatible router.
- They are profile names on that gateway, not a separate `api.clues.*` host.

## Best current defaults

1. Coding: Hugging Face `Qwen/Qwen3-Coder-Next:fastest`
2. Low-latency general work: OpenRouter `openrouter/auto`
3. Balanced fallback: GitHub Models `openai/gpt-4o-mini`
4. Long-form reasoning: Hugging Face `deepseek-ai/DeepSeek-R1:fastest`

## Working GitHub Models

- `openai/gpt-4.1-mini`
- `openai/gpt-4o-mini`
- `meta/Llama-3.3-70B-Instruct`

Observed limit:

- `deepseek/DeepSeek-V3-0324` is valid but often rate-limited with `429`
- On April 3, 2026, the tested GitHub free models worked in a minimal workspace but rejected the full Clues startup payload in this repo once the total request crossed the current ~8k token request-size cap.

## Working OpenRouter models

- `openrouter/auto`

Observed limit:

- many `:free` models returned `429` during testing, including some NVIDIA and Qwen free variants

## Working Hugging Face router models

- `Qwen/Qwen3-Coder-Next:fastest`
- `deepseek-ai/DeepSeek-R1:fastest`
- `openai/gpt-oss-20b:fastest`
- `Qwen/Qwen3.5-27B:fastest`
- `google/gemma-4-31B-it:fastest`
- `meta-llama/Llama-3.1-8B-Instruct:fastest`
- `zai-org/GLM-5:fastest`
- `MiniMaxAI/MiniMax-M2.5:fastest`

## Recommended CLI alias mapping

- `fast`
  - OpenRouter `openrouter/auto`

- `code`
  - Hugging Face `Qwen/Qwen3-Coder-Next:fastest`

- `reasoner`
  - Hugging Face `deepseek-ai/DeepSeek-R1:fastest`

- `github`
  - GitHub Models `openai/gpt-4o-mini`

## Operating guidance

- Prefer Hugging Face or OpenRouter for day-to-day coding latency.
- Use GitHub Models as a strong additional pool, but expect free-tier throttling and smaller request budgets on the currently tested free models.
- If you deploy the optional gateway, keep its profile names neutral and map them onto these provider defaults.
