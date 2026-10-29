<p align="center"><strong>Apeiron Agents</strong> is a coding agent that runs locally on your computer.</p>

---

Apeiron Agents is a fork of [OpenAI Codex CLI](https://github.com/openai/codex), maintained by Apeiron Intelligence and distributed under the Apache 2.0 licence. It is **not** an OpenAI product and is not endorsed by, affiliated with, or supported by OpenAI. See [NOTICE](NOTICE) for attribution and the list of upstream identifiers this fork deliberately keeps.

## Quickstart

### Building from source

This fork is not published to a package registry. Build it from a checkout:

```shell
git clone https://github.com/VoicuTomut/apeiron-codex.git
cd apeiron-codex/codex-rs
cargo build --release -p codex-cli
```

The binary is written to `codex-rs/target/release/codex`. See [Installing & building](./docs/install.md) for prerequisites and platform notes.

> **Note:** upstream's installers (`chatgpt.com/codex/install.sh`, `npm install -g @openai/codex`, `brew install --cask codex`) install **OpenAI's** Codex CLI, not this fork. Do not use them to install Apeiron Agents.

### Signing in

Apeiron Agents authenticates against OpenAI's API, because it uses OpenAI models. Run the binary and select **Sign in with ChatGPT**, or supply an API key. Your usage is governed by your own agreement with OpenAI; this fork's licence covers the software only, not model access.

## Docs

- [**Contributing**](./docs/contributing.md)
- [**Installing & building**](./docs/install.md)
- [**Configuration**](./docs/config.md)
- [**Getting started**](./docs/getting-started.md)

## Licence

This repository is licensed under the [Apache-2.0 License](LICENSE), inherited from the upstream project. Commercial use, modification and redistribution are permitted; see [NOTICE](NOTICE) for the attribution and change-notice requirements that come with it.
