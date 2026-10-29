# Security Policy

## Reporting Security Issues

Apeiron Agents is a fork of OpenAI Codex CLI, maintained by Apeiron Intelligence. **Do not report vulnerabilities in this fork to OpenAI.** OpenAI's Bugcrowd program covers OpenAI's own software and will not accept reports about code we maintain.

Report vulnerabilities in Apeiron Agents privately, via [GitHub private vulnerability reporting](https://github.com/VoicuTomut/apeiron-codex/security/advisories/new) on this repository. Please do not open a public issue for a security problem.

If you can reproduce the vulnerability against an **unmodified upstream** Codex CLI, it is an upstream issue: report it through OpenAI's [Bugcrowd program](https://bugcrowd.com/engagements/openai) so it reaches the people who can fix it at the source. Telling us as well is welcome, so we can track whether this fork is affected.

## How to operate Apeiron Agents safely

Apeiron Agents inherits its security boundaries — sandboxing, approvals, and network controls — from upstream Codex CLI. For details on how that model works, see the [upstream documentation](https://developers.openai.com/codex/agent-approvals-security). Note that upstream documentation describes upstream's behaviour; where this fork diverges, this repository is the authority.
