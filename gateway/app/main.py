import copy
import os
from dataclasses import dataclass
from typing import Any

import httpx
from fastapi import FastAPI, HTTPException, Request
from fastapi.responses import JSONResponse, StreamingResponse
from starlette.background import BackgroundTask

DEFAULT_TIMEOUT_SECONDS = float(os.getenv("CLUES_TIMEOUT_SECONDS", "45"))
DEFAULT_MAX_TOKENS = int(os.getenv("CLUES_MAX_TOKENS", "4096"))

OPENROUTER_BASE_URL = os.getenv("OPENROUTER_BASE_URL", "https://openrouter.ai/api/v1")
HUGGINGFACE_BASE_URL = os.getenv(
    "HUGGINGFACE_BASE_URL", "https://router.huggingface.co/v1"
)
GITHUB_MODELS_BASE_URL = os.getenv(
    "GITHUB_MODELS_BASE_URL", "https://models.github.ai/inference"
)

KNOWN_GITHUB_MODELS = [
    "openai/gpt-4.1-mini",
    "openai/gpt-4o-mini",
    "meta/Llama-3.3-70B-Instruct",
    "deepseek/DeepSeek-V3-0324",
]

KNOWN_OPENROUTER_MODELS = [
    "openrouter/auto",
    "openrouter/free",
    "qwen/qwen3.6-plus:free",
    "deepseek/deepseek-r1:free",
    "nvidia/nemotron-3-super-120b-a12b:free",
    "nvidia/nemotron-3-nano-30b-a3b:free",
]

KNOWN_HUGGINGFACE_MODELS = [
    "Qwen/Qwen3-Coder-Next:fastest",
    "deepseek-ai/DeepSeek-R1:fastest",
    "openai/gpt-oss-20b:fastest",
    "Qwen/Qwen3.5-27B:fastest",
    "google/gemma-4-31B-it:fastest",
    "meta-llama/Llama-3.1-8B-Instruct:fastest",
    "zai-org/GLM-5:fastest",
    "MiniMaxAI/MiniMax-M2.5:fastest",
]


@dataclass(frozen=True)
class Candidate:
    provider: str
    model: str
    label: str


PROFILE_MODELS: dict[str, list[Candidate]] = {
    "clues/coder-fast": [
        Candidate("openrouter", "openrouter/auto", "OpenRouter auto"),
        Candidate(
            "huggingface",
            "Qwen/Qwen3-Coder-Next:fastest",
            "Hugging Face Qwen coder",
        ),
        Candidate("github", "openai/gpt-4.1-mini", "GitHub GPT-4.1 mini"),
    ],
    "clues/coder-balanced": [
        Candidate(
            "huggingface",
            "Qwen/Qwen3-Coder-Next:fastest",
            "Hugging Face Qwen coder",
        ),
        Candidate("openrouter", "openrouter/auto", "OpenRouter auto"),
        Candidate("github", "openai/gpt-4o-mini", "GitHub GPT-4o mini"),
    ],
    "clues/reasoner": [
        Candidate(
            "huggingface",
            "deepseek-ai/DeepSeek-R1:fastest",
            "Hugging Face DeepSeek R1",
        ),
        Candidate("openrouter", "openrouter/auto", "OpenRouter auto"),
        Candidate("github", "deepseek/DeepSeek-V3-0324", "GitHub DeepSeek V3"),
    ],
    "clues/github-fast": [
        Candidate("github", "openai/gpt-4.1-mini", "GitHub GPT-4.1 mini"),
        Candidate("github", "openai/gpt-4o-mini", "GitHub GPT-4o mini"),
        Candidate(
            "github",
            "meta/Llama-3.3-70B-Instruct",
            "GitHub Llama 3.3 70B",
        ),
    ],
}

app = FastAPI(title="Clues Gateway", version="0.1.0")


def non_empty_env(*names: str) -> str | None:
    for name in names:
        value = os.getenv(name, "").strip()
        if value:
            return value
    return None


def require_gateway_auth(request: Request) -> None:
    expected = non_empty_env("CLUES_GATEWAY_TOKEN")
    if not expected:
        return
    auth_header = request.headers.get("authorization", "").strip()
    token = auth_header.removeprefix("Bearer").strip()
    if token != expected:
        raise HTTPException(status_code=401, detail="invalid gateway token")


def provider_key(provider: str) -> str | None:
    if provider == "openrouter":
        return non_empty_env("OPENROUTER_API_KEY")
    if provider == "huggingface":
        return non_empty_env("HF_TOKEN", "HUGGINGFACE_API_KEY")
    if provider == "github":
        return non_empty_env("GITHUB_TOKEN")
    return None


def provider_base_url(provider: str) -> str:
    if provider == "openrouter":
        return OPENROUTER_BASE_URL.rstrip("/")
    if provider == "huggingface":
        return HUGGINGFACE_BASE_URL.rstrip("/")
    if provider == "github":
        return GITHUB_MODELS_BASE_URL.rstrip("/")
    raise ValueError(f"unsupported provider: {provider}")


def provider_headers(provider: str) -> dict[str, str]:
    api_key = provider_key(provider)
    if not api_key:
        raise HTTPException(status_code=503, detail=f"{provider} is not configured")
    headers = {
        "authorization": f"Bearer {api_key}",
        "content-type": "application/json",
    }
    if provider == "openrouter":
        headers["http-referer"] = os.getenv(
            "CLUES_PUBLIC_URL", "https://railway.app"
        ).strip()
        headers["x-title"] = "Clues Code Gateway"
    return headers


def available_provider(provider: str) -> bool:
    return provider_key(provider) is not None


def visible_models() -> list[dict[str, Any]]:
    models: list[dict[str, Any]] = []
    for profile_id, candidates in PROFILE_MODELS.items():
        if any(available_provider(candidate.provider) for candidate in candidates):
            models.append(
                {
                    "id": profile_id,
                    "object": "model",
                    "owned_by": "clues-gateway",
                }
            )
    if available_provider("github"):
        for model in KNOWN_GITHUB_MODELS:
            models.append({"id": f"github::{model}", "object": "model", "owned_by": "github"})
    if available_provider("openrouter"):
        for model in KNOWN_OPENROUTER_MODELS:
            models.append(
                {"id": f"openrouter::{model}", "object": "model", "owned_by": "openrouter"}
            )
    if available_provider("huggingface"):
        for model in KNOWN_HUGGINGFACE_MODELS:
            models.append(
                {
                    "id": f"huggingface::{model}",
                    "object": "model",
                    "owned_by": "huggingface",
                }
            )
    return models


def resolve_candidates(model: str) -> list[Candidate]:
    requested = model.strip()
    if not requested:
        raise HTTPException(status_code=400, detail="missing model")

    if requested in PROFILE_MODELS:
        candidates = [
            candidate
            for candidate in PROFILE_MODELS[requested]
            if available_provider(candidate.provider)
        ]
        if not candidates:
            raise HTTPException(
                status_code=503,
                detail=f"no configured providers are available for profile {requested}",
            )
        return candidates

    if "::" in requested:
        provider, raw_model = requested.split("::", 1)
        provider = provider.strip().lower()
        raw_model = raw_model.strip()
        if provider not in {"github", "openrouter", "huggingface"}:
            raise HTTPException(status_code=400, detail=f"unsupported provider prefix: {provider}")
        if not available_provider(provider):
            raise HTTPException(status_code=503, detail=f"{provider} is not configured")
        return [Candidate(provider, raw_model, f"{provider} explicit model")]

    if requested in KNOWN_GITHUB_MODELS and available_provider("github"):
        return [Candidate("github", requested, "GitHub direct model")]
    if (
        requested in KNOWN_OPENROUTER_MODELS
        or requested.startswith("openrouter/")
        or requested.endswith(":free")
    ) and available_provider("openrouter"):
        return [Candidate("openrouter", requested, "OpenRouter direct model")]
    if (
        requested in KNOWN_HUGGINGFACE_MODELS
        or requested.endswith(":fastest")
        or requested.endswith(":cheapest")
        or requested.endswith(":preferred")
    ) and available_provider("huggingface"):
        return [Candidate("huggingface", requested, "Hugging Face direct model")]

    return resolve_candidates("clues/coder-fast")


def normalize_request(body: dict[str, Any], candidate: Candidate) -> dict[str, Any]:
    payload = copy.deepcopy(body)
    payload["model"] = candidate.model
    if "max_tokens" not in payload and "max_completion_tokens" not in payload:
        payload["max_tokens"] = DEFAULT_MAX_TOKENS
    if candidate.provider in {"github", "huggingface"} and isinstance(
        payload.get("max_tokens"), int
    ):
        payload["max_tokens"] = min(payload["max_tokens"], DEFAULT_MAX_TOKENS)
    return payload


def format_error_body(body: str) -> str:
    body = body.strip()
    if len(body) <= 400:
        return body
    return f"{body[:400]}..."


def gateway_headers(candidate: Candidate, attempt: int, total: int) -> dict[str, str]:
    return {
        "x-clues-provider": candidate.provider,
        "x-clues-model": candidate.model,
        "x-clues-attempt": f"{attempt}/{total}",
    }


async def close_stream(response: httpx.Response, client: httpx.AsyncClient) -> None:
    await response.aclose()
    await client.aclose()


async def try_json_candidate(
    candidate: Candidate, payload: dict[str, Any], attempt: int, total: int
) -> tuple[JSONResponse | None, dict[str, Any] | None]:
    async with httpx.AsyncClient(timeout=DEFAULT_TIMEOUT_SECONDS) as client:
        response = await client.post(
            f"{provider_base_url(candidate.provider)}/chat/completions",
            headers=provider_headers(candidate.provider),
            json=payload,
        )
        body = response.text
        if response.is_success:
            return (
                JSONResponse(
                    content=response.json(),
                    status_code=response.status_code,
                    headers=gateway_headers(candidate, attempt, total),
                ),
                None,
            )
        return None, {
            "provider": candidate.provider,
            "model": candidate.model,
            "status": response.status_code,
            "body": format_error_body(body),
        }


async def try_stream_candidate(
    candidate: Candidate, payload: dict[str, Any], attempt: int, total: int
) -> tuple[StreamingResponse | None, dict[str, Any] | None]:
    client = httpx.AsyncClient(timeout=None)
    request = client.build_request(
        "POST",
        f"{provider_base_url(candidate.provider)}/chat/completions",
        headers=provider_headers(candidate.provider),
        json=payload,
    )
    response = await client.send(request, stream=True)
    if response.is_success:
        media_type = response.headers.get("content-type", "text/event-stream")
        return (
            StreamingResponse(
                response.aiter_raw(),
                status_code=response.status_code,
                media_type=media_type,
                headers=gateway_headers(candidate, attempt, total),
                background=BackgroundTask(close_stream, response, client),
            ),
            None,
        )
    body = (await response.aread()).decode("utf-8", errors="replace")
    await response.aclose()
    await client.aclose()
    return None, {
        "provider": candidate.provider,
        "model": candidate.model,
        "status": response.status_code,
        "body": format_error_body(body),
    }


@app.get("/health")
async def health() -> dict[str, Any]:
    return {
        "ok": True,
        "providers": {
            "github": available_provider("github"),
            "openrouter": available_provider("openrouter"),
            "huggingface": available_provider("huggingface"),
        },
    }


@app.get("/v1/models")
async def list_models(request: Request) -> dict[str, Any]:
    require_gateway_auth(request)
    return {"object": "list", "data": visible_models()}


@app.post("/v1/chat/completions")
async def chat_completions(request: Request) -> JSONResponse | StreamingResponse:
    require_gateway_auth(request)
    body = await request.json()
    if not isinstance(body, dict):
        raise HTTPException(status_code=400, detail="request body must be a JSON object")

    requested_model = str(body.get("model", "")).strip()
    candidates = resolve_candidates(requested_model)
    wants_stream = bool(body.get("stream"))
    failures: list[dict[str, Any]] = []

    for index, candidate in enumerate(candidates, start=1):
        payload = normalize_request(body, candidate)
        if wants_stream:
            stream_response, failure = await try_stream_candidate(
                candidate, payload, index, len(candidates)
            )
            if stream_response is not None:
                return stream_response
            if failure is not None:
                failures.append(failure)
                continue
        else:
            json_response, failure = await try_json_candidate(
                candidate, payload, index, len(candidates)
            )
            if json_response is not None:
                return json_response
            if failure is not None:
                failures.append(failure)
                continue

    return JSONResponse(
        status_code=502,
        content={
            "error": {
                "message": f"all candidates failed for model {requested_model}",
                "type": "gateway_provider_failure",
                "attempts": failures,
            }
        },
    )
