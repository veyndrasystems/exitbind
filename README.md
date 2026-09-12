# Soulmate

It verifies what you asked an agent to do and what came back, in the same record.

Soulmate adds a bounded, resumable acceptance record to the coding agent you
already use. You keep talking to your existing Codex or Claude lead. For work
where independent review, recovery, or exact check evidence matters, the lead
uses Soulmate automatically; small reversible work stays in the conversation.
Soulmate is not another model, agent runtime, daemon, or cloud service.

[![Rust primary CI](https://github.com/veyndrasystems/soulmate/actions/workflows/ci.yml/badge.svg)](https://github.com/veyndrasystems/soulmate/actions/workflows/ci.yml)
[![Stable release](https://img.shields.io/github/v/release/veyndrasystems/soulmate)](https://github.com/veyndrasystems/soulmate/releases/latest)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

## Start in your existing conversation

Give your current coding agent the repository once:

> Set up Soulmate for this project from https://github.com/veyndrasystems/soulmate. Inspect first, ask before installing, writing setup files, or changing permissions, then use it yourself when appropriate. Keep small reversible work direct.

The lead inspects first and asks only for real installation, project-write, or
permission decisions. After approval, it manages setup and later Soulmate work
inside the same root conversation. You do not need to learn commands, select a
workflow, copy JSON, or carry ledger paths, event hashes, artifact paths, check
results, handles, or recovery choices.

Unless the project requires a stable-only channel, the lead should propose the
pinned `v0.16.0-rc.1` preview and clearly name it as a prerelease. Stable
`0.12.0` remains an explicit alternative with its
[matching documentation](https://github.com/veyndrasystems/soulmate/tree/349b662574b29a2b0366f53aac12d97f268bc84c).

After setup, ordinary requests stay ordinary:

> Please update the theme, run the existing checks, and tell me what changed and what still needs doing.

At the start of a material task, the lead makes one lightweight choice. It uses
Soulmate when bounded delegation, independent review, resumability,
deterministic handoff/check evidence, false-completion protection, or a risky
migration materially helps. A typo or tiny reversible edit remains direct.

These states are deliberately different:

| State | What it establishes |
| --- | --- |
| Available | Soulmate exists in the project or environment. |
| Discovered | This agent session can see its project guidance. |
| Selected | The lead chose governed handling for this task. |
| Activated | A successful `work begin` created machine-observable work and returned the next action. |

Projected files and setup text do not prove discovery, future agent compliance,
selection, or activation. If governed handling materially matters but activation
is unavailable, the lead stops before scoped implementation and reports the
failing layer instead of silently falling back.

## Install the preview and see the refusal

This page describes `v0.16.0-rc.1` for Linux x86_64 and macOS on Apple Silicon
or Intel. The pinned installer writes one executable under `$HOME/.local/bin`
and verifies its archive checksum. Review the command and destination before
approving installation. When repository provenance is required before install,
follow the [GitHub attestation verification](REFERENCE.md#conversational-update-notice).

```sh
curl -fsSL https://raw.githubusercontent.com/veyndrasystems/soulmate/v0.16.0-rc.1/install.sh | sh
export PATH="$HOME/.local/bin:$PATH"
```

Run the setup-free, model-free experiment from any directory:

```sh
soulmate benchmark
```

Success starts with `False-completion proof passed (14/14 assertions).` The
fixture uses real local command exit codes and synthetic actors to show:

```text
Attempt 1 failed its current check, so acceptance was refused.
Rework preserved that attempt; a fresh checked attempt still needed review and lead acceptance.
```

In checked runs, Soulmate refuses acceptance when the configured check result is missing or reports failure for the current worker artifact.

The benchmark uses a disposable Git project, leaves the current worktree
untouched, and removes its records by default. Use `soulmate benchmark --output
NEW_DIRECTORY` to keep them. This proves the observed refusal and recovery
path—not general time, token, quality, adoption, or human-behavior outcomes.
See the [proof methodology](docs/value-proof-methodology.md).

## What approved project setup creates

For manual setup, run this from the project directory after approving its
writes:

```sh
soulmate init --mode portable --root .
```

`init` writes reviewable `soulmate.json`, private ignored state under
`.soulmate/`, role profiles, and exact project-local Soulmate skill copies for
Codex and Claude. It installs no hook, launches no agent, and grants no host or
OS permission. Existing managed skill bytes that differ are never silently
overwritten; inspect them and use the explicit refresh command printed by the
CLI.

Setup reports each layer honestly: binary availability, skill projection,
unverified fresh- and active-session discovery, task selection not yet
performed, and activation not yet performed. Review the generated task
boundaries and native worker/reviewer mapping before project-scoped work. Empty
starter boundaries are valid configuration, not permission to read, write, or
run project commands.

Codex and Claude are generated setup paths. OpenCode is documented as
path-compatible through `.agents/skills/soulmate/`, but Soulmate CI does not
exercise its host execution. Other hosts retain their own discovery and consent
contracts. See [host onboarding](docs/onboarding.md#keep-the-host-you-already-use).

## How governed work proceeds

The lead uses the high-level `work` surface; the human does not:

```text
normal request
  -> classify once
  -> small and reversible: work directly
  -> governed: work begin -> follow one returned action at a time
       -> worker result -> frozen check -> independent review -> lead decision
```

`work begin` creates a checked run and returns an opaque work handle plus one
validated next action. `work return` stores role results in managed artifacts,
`work check` executes and binds the frozen local policy to the current worker,
and `work resume` reports zero, one, or multiple active work items without
guessing. The façade hides routine ledger/event/target/check bookkeeping from
the lead while preserving it in the strict record.

The [matched control-surface evaluation](docs/agent-first-evaluation.md)
executes the same strict checked workflow through both interfaces and reports
only directly observed protocol-transport differences.

Worker completion, a passing check, reviewer approval, and lead acceptance stay
separate. A missing or failed check blocks acceptance. Rework preserves the old
attempt and requires a fresh worker result, check, review, and decision. Artifact
byte drift or configuration/profile/boundary drift stops mutation rather than
inventing continuity.

Low-level `run` commands remain the advanced interoperability and inspection
surface. Use [the complete checked example](docs/first-checked-run.md),
[repair and resumption guide](docs/repair-a-run.md), or
[command reference](REFERENCE.md) when manual control is actually needed.

## Trust, privacy, and compatibility

Soulmate relies on the existing host for model execution, native subagents, and
permissions. It does not authenticate a model's self-report or make local
evidence tamper-proof against a process that can rewrite all project state.
Keep raw ledgers and artifacts private: they can contain goals, commands,
paths, and task results. Read [SECURITY.md](SECURITY.md) before real work.

The `v0.16.0-rc.1` preview adds an agent-first façade and project selection
contract without changing persisted run-event formats. New checked runs still
use v4; v3 remains readable. Existing low-level commands and v1–v4 readers stay
available. In the public check mapping, v3 supports caller-reported `record-check` only; v4 supports both reported `record-check` and observed `observe-check`. See the
[public format map](CHANGELOG.md#public-tags-and-format-readers) before choosing
a rollback or release channel.

External contributors do not need Soulmate. Normal forks, patches, tests, and
pull requests remain welcome through [CONTRIBUTING.md](CONTRIBUTING.md).

- [Host onboarding](docs/onboarding.md) · [First checked run](docs/first-checked-run.md)
- [Repair or resume](docs/repair-a-run.md) · [Terminology](docs/glossary.md)
- [Optional memory, hooks, and receipts](docs/optional-surfaces.md)
- [Windows and WSL 2](docs/windows-wsl.md) · [Platform support](docs/platform-support.md)
- [Usage findings](docs/participation-validation.md) · [Claim registry](proof/claims.json)
- [Contributing](CONTRIBUTING.md) · [Security](SECURITY.md) · [License](LICENSE)
