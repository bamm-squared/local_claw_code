# Run Local / Providers

`claw` can now target multiple providers instead of assuming Anthropic-only credentials.
You can select a hosted provider with an API key, point at a compatible gateway, or run against a local model server such as Ollama.

This guide covers the high-level setup for:

- Anthropic / Claude
- OpenAI / ChatGPT
- Google Gemini
- xAI / Grok
- OpenAI-compatible endpoints
- Anthropic-compatible endpoints
- Ollama and local models

## Core concepts

There are three main ways to choose a backend:

1. Pass flags on the command line.
2. Store defaults in config.
3. Let provider detection infer a backend from the model name when that is unambiguous.

The most explicit form is:

```bash
claw --provider PROVIDER --model MODEL "your prompt"
```

Useful flags:

- `--provider` selects the provider explicitly
- `--provider-base-url` overrides the endpoint for compatible gateways or custom hosts
- `--model` selects the model name that your provider exposes

Useful health checks:

- `claw doctor`
- `claw status`
- `claw /config provider`

## Supported provider ids

These provider ids are supported directly:

- `anthropic`
- `openai`
- `xai`
- `gemini`
- `openai-compatible`
- `anthropic-compatible`
- `ollama`

## Quick examples

Anthropic:

```bash
export ANTHROPIC_API_KEY="..."
claw --provider anthropic --model claude-sonnet-4-6 "summarize this repo"
```

OpenAI:

```bash
export OPENAI_API_KEY="..."
claw --provider openai --model gpt-5 "summarize this repo"
```

Gemini:

```bash
export GEMINI_API_KEY="..."
claw --provider gemini --model gemini-2.5-pro "summarize this repo"
```

xAI:

```bash
export XAI_API_KEY="..."
claw --provider xai --model grok-3 "summarize this repo"
```

Ollama:

```bash
claw --provider ollama --model granite4:3b "summarize this repo"
```

OpenAI-compatible gateway:

```bash
export OPENAI_COMPAT_API_KEY="..."
claw --provider openai-compatible \
  --provider-base-url https://your-host.example/v1 \
  --model your-model \
  "summarize this repo"
```

Anthropic-compatible gateway:

```bash
export ANTHROPIC_COMPAT_AUTH_TOKEN="..."
claw --provider anthropic-compatible \
  --provider-base-url https://your-host.example \
  --model your-model \
  "summarize this repo"
```

## Environment variables by provider

Hosted providers:

- Anthropic: `ANTHROPIC_API_KEY` or `ANTHROPIC_AUTH_TOKEN`
- OpenAI: `OPENAI_API_KEY`
- Gemini: `GEMINI_API_KEY`
- xAI: `XAI_API_KEY`

Compatible providers:

- OpenAI-compatible: `OPENAI_COMPAT_API_KEY`, `OPENAI_COMPAT_BASE_URL`
- Anthropic-compatible: `ANTHROPIC_COMPAT_AUTH_TOKEN` or `ANTHROPIC_COMPAT_API_KEY`, `ANTHROPIC_COMPAT_BASE_URL`

Local Ollama defaults:

- Base URL defaults to `http://localhost:11434`
- Auth is handled automatically with a local compatibility token fallback
- Optional overrides: `OLLAMA_BASE_URL`, `OLLAMA_AUTH_TOKEN`, `OLLAMA_API_KEY`

## Configuring a default provider

You can store a default model and provider in your Claw config.

String form:

```json
{
  "model": "gpt-5",
  "provider": "openai"
}
```

Object form with an explicit base URL:

```json
{
  "model": "granite4:3b",
  "provider": {
    "id": "ollama",
    "baseUrl": "http://localhost:11434"
  }
}
```

You can inspect the merged result with:

```bash
claw /config provider
```

## Running with Ollama

Start Ollama normally and make sure the model you want is installed:

```bash
ollama list
ollama run granite4:3b
```

Then point `claw` at Ollama:

```bash
claw --provider ollama --model granite4:3b "summarize this repository"
```

For agentic or tool-driven use, choose a model that actually supports tools well in Ollama.
Some smaller local models are fine for chat but unreliable for file reads, edits, or tool loops.

A practical workflow is:

```bash
claw --provider ollama --model granite4:3b --allowedTools read_file "inspect Cargo.toml and summarize it"
```

If your Ollama instance is on another host or port:

```bash
claw --provider ollama \
  --provider-base-url http://192.168.1.50:11434 \
  --model granite4:3b \
  "summarize this repository"
```

## Running with hosted providers

For hosted providers, export the matching API key first and then select the provider explicitly.

Examples:

```bash
export OPENAI_API_KEY="..."
claw --provider openai --model gpt-5 "review the latest diff"
```

```bash
export GEMINI_API_KEY="..."
claw --provider gemini --model gemini-2.5-pro "explain this codebase"
```

```bash
export XAI_API_KEY="..."
claw --provider xai --model grok-3 "write release notes"
```

If you prefer Anthropic OAuth, that flow still exists:

```bash
claw login
```

Important:

- `claw login` and `claw logout` are Anthropic-only
- Other providers are configured with env vars and optional base URL overrides

## Multi-agent and local models

The multi-agent workflow is local to the CLI runtime.
Each agent makes its own model requests through the configured provider.

That means local backends can work for multi-agent runs too, as long as:

- the model supports the needed tool-calling behavior
- the backend can handle concurrent requests
- your machine has enough RAM or VRAM for the number of active agents

Ollama can work here, but the quality of the experience depends heavily on the model you choose.

## Troubleshooting

If `claw` does not behave the way you expect:

1. Run `claw doctor`
2. Run `claw /config provider`
3. Make sure the model name exists on the selected provider
4. Make sure the provider-specific API key or base URL is set
5. For local models, confirm the model supports tools if you want full agent behavior

Common cases:

- Plain chat works, but tool use fails:
  Your model likely does not support tool calling well enough for agentic use.
- The provider is wrong:
  Pass `--provider` explicitly instead of relying on model inference.
- A compatible endpoint fails:
  Set `--provider-base-url` and use the matching compat provider id.

## Recommended starting points

If you want the least ambiguity, start with explicit flags:

```bash
claw --provider anthropic --model claude-sonnet-4-6 "summarize this repo"
claw --provider openai --model gpt-5 "summarize this repo"
claw --provider gemini --model gemini-2.5-pro "summarize this repo"
claw --provider xai --model grok-3 "summarize this repo"
claw --provider ollama --model granite4:3b "summarize this repo"
```

Once that works, move the provider and model into config so each repo opens with the backend you actually want.
