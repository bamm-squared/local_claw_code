# Note:

Thanks to the team who did the original work on this. This repo modifies ultraworkers/claw-code to make it not provider locked to Anthropic.

# Claw Code

<p align="center">
  <a href="https://github.com/ultraworkers/claw-code">ultraworkers/claw-code</a>
  &middot;
  <a href="./USAGE.md">Usage</a>
  &middot;
  <a href="./RUNLOCAL.md">Run Local / Providers</a>
  &middot;
  <a href="./rust/README.md">Rust workspace</a>
  &middot;
  <a href="./PARITY.md">Parity</a>
  &middot;
  <a href="./ROADMAP.md">Roadmap</a>
  &middot;
  <a href="https://discord.gg/5TUQKqFWd">UltraWorkers Discord</a>
</p>

<p align="center">
  <a href="https://star-history.com/#ultraworkers/claw-code&Date">
    <picture>
      <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/svg?repos=ultraworkers/claw-code&type=Date&theme=dark" />
      <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/svg?repos=ultraworkers/claw-code&type=Date" />
      <img alt="Star history for ultraworkers/claw-code" src="https://api.star-history.com/svg?repos=ultraworkers/claw-code&type=Date" width="600" />
    </picture>
  </a>
</p>

<p align="center">
  <img src="assets/claw-hero.jpeg" alt="Claw Code" width="300" />
</p>

Claw Code is the public Rust implementation of the `claw` CLI agent harness.
The canonical implementation lives in [`rust/`](./rust), and the current source of truth for this repository is **ultraworkers/claw-code**.

Claw can now run against multiple hosted providers or local model backends instead of assuming Anthropic-only credentials.

> [!IMPORTANT]
> Start with [`USAGE.md`](./USAGE.md) for build, auth, CLI, session, and parity-harness workflows. Use [`RUNLOCAL.md`](./RUNLOCAL.md) for provider selection, Ollama/local-model setup, and API-key-based hosted providers. Make `claw doctor` your first health check after building, use [`rust/README.md`](./rust/README.md) for crate-level details, read [`PARITY.md`](./PARITY.md) for the current Rust-port checkpoint, and see [`docs/container.md`](./docs/container.md) for the container-first workflow.

## Provider support

Claw can be configured per command or in config to talk to:

- Anthropic / Claude
- OpenAI / ChatGPT
- Google Gemini
- xAI / Grok
- OpenAI-compatible gateways
- Anthropic-compatible gateways
- Local Ollama-hosted models

High level:

- Use `--provider` to pick a backend explicitly.
- Use `--provider-base-url` to override the endpoint when needed.
- Use config to make a provider and model pair the default for a repo or machine.
- `claw login` and `claw logout` are only for Anthropic OAuth. Other providers use API keys or local endpoints.
- `claw init` now scaffolds an `AGENTS.md` instruction file while still honoring legacy `CLAUDE.md` files.

See [`RUNLOCAL.md`](./RUNLOCAL.md) for concrete setup examples.

## Current repository shape

- **`rust/`** - canonical Rust workspace and the `claw` CLI binary
- **`USAGE.md`** - task-oriented usage guide for the current product surface
- **`RUNLOCAL.md`** - provider setup for Ollama, hosted APIs, and compatible endpoints
- **`PARITY.md`** - Rust-port parity status and migration notes
- **`ROADMAP.md`** - active roadmap and cleanup backlog
- **`PHILOSOPHY.md`** - project intent and system-design framing
- **`src/` + `tests/`** - companion Python/reference workspace and audit helpers; not the primary runtime surface

## Quick start

```bash
cd rust
cargo build --workspace
./target/debug/claw --help
./target/debug/claw prompt "summarize this repository"
```

Choose a provider and authenticate with either an API key, Anthropic OAuth, or a local backend:

```bash
export ANTHROPIC_API_KEY="sk-ant-..."
# or
cd rust
./target/debug/claw login
```

You can also run against another hosted provider or Ollama:

```bash
./target/debug/claw --provider openai --model gpt-5 "summarize this repository"
./target/debug/claw --provider ollama --model granite4:3b "summarize this repository"
```

For provider-specific environment variables, config examples, and local-model notes, see [`RUNLOCAL.md`](./RUNLOCAL.md).

Run the workspace test suite:

```bash
cd rust
cargo test --workspace
```

## Documentation map

- [`USAGE.md`](./USAGE.md) - quick commands, auth, sessions, config, parity harness
- [`RUNLOCAL.md`](./RUNLOCAL.md) - provider selection, local models, API keys, and endpoint overrides
- [`rust/README.md`](./rust/README.md) - crate map, CLI surface, features, workspace layout
- [`PARITY.md`](./PARITY.md) - parity status for the Rust port
- [`rust/MOCK_PARITY_HARNESS.md`](./rust/MOCK_PARITY_HARNESS.md) - deterministic mock-service harness details
- [`ROADMAP.md`](./ROADMAP.md) - active roadmap and open cleanup work
- [`PHILOSOPHY.md`](./PHILOSOPHY.md) - why the project exists and how it is operated

## Ecosystem

Claw Code is built in the open alongside the broader UltraWorkers toolchain:

- [clawhip](https://github.com/Yeachan-Heo/clawhip)
- [oh-my-openagent](https://github.com/code-yeongyu/oh-my-openagent)
- [oh-my-claudecode](https://github.com/Yeachan-Heo/oh-my-claudecode)
- [oh-my-codex](https://github.com/Yeachan-Heo/oh-my-codex)
- [UltraWorkers Discord](https://discord.gg/5TUQKqFWd)

## Ownership / affiliation disclaimer

- This repository does **not** claim ownership of the original Claude Code source material.
- This repository is **not affiliated with, endorsed by, or maintained by Anthropic**.
