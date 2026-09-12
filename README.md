# Soulmate

It verifies what you asked an agent to do and what came back, in the same record.

Soulmate adds a durable, inspectable thread around the coding agent you already
use. Keep the goal, the work that was submitted, the check for the current
attempt, review, and decision together. When work needs a bounded handoff,
rework, or a resumable next assignment, the record makes the trail easy to
follow. Simple, reversible work can stay in the direct conversation.

[![Rust primary CI](https://github.com/veyndrasystems/soulmate/actions/workflows/ci.yml/badge.svg)](https://github.com/veyndrasystems/soulmate/actions/workflows/ci.yml)
[![Stable release](https://img.shields.io/github/v/release/veyndrasystems/soulmate)](https://github.com/veyndrasystems/soulmate/releases/latest)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

## Start with your existing agent

Keep using your existing Codex or Claude lead. Describe the work in ordinary
language in that same root conversation. The lead inspects the repository and
host first. Ask before either installation or a host-permission change; setup
writes also require that approval. After approval, the lead manages the
authorized setup and protocol in that root conversation.
Ordinary reversible work stays direct.

Unless the project already requires a non-prerelease channel, the lead proposes
the pinned current `v0.15.0-rc.3` preview and names it as a prerelease in the
install approval. The human does not need to choose a channel first. Stable
`0.12.0` remains opt-in when that requirement is stated; use its matching
documentation.

Use this short first request for the current project:

> Set up Soulmate for this project from https://github.com/veyndrasystems/soulmate. Inspect first, ask before installing, writing setup files, or changing permissions, then manage it yourself and keep simple work direct.

Use Soulmate for bounded delegation, independent review, or resumable work;
skip it for a typo or small reversible edit. You own the intended outcome,
meaningful preferences, permissions, and decisions; silence is never approval.
The existing host owns models, execution, and permissions. Soulmate owns its
bounded records and lifecycle checks.

After setup, a normal-language request can remain simple:

> Please update the theme, run the existing checks, and tell me what changed
> and what still needs doing.

The root conversation stays the place where the human and lead coordinate. A
bounded worker can claim completion, but the current attempt still needs its
agreed check. A missing or failed check blocks acceptance; reviewer approval is
not the lead's acceptance. Rework keeps earlier attempts available, and the
host can retrieve the next validated assignment without inventing a new thread.

## Install and see the result

This page describes the current preview, `v0.15.0-rc.3`, for Linux x86_64 and
macOS on Apple Silicon or Intel. The stable `0.12.0` channel remains available
through the [stable release documentation](https://github.com/veyndrasystems/soulmate/tree/349b662574b29a2b0366f53aac12d97f268bc84c);
preview and stable binaries are separate channels. The pinned installer writes one executable under
`$HOME/.local/bin` and verifies its archive checksum. Your existing host still
provides execution and permissions; `init` later writes reviewable project
configuration and generated skill copies.

Before installing or using it, review the pinned command and destination. The
benchmark uses no model or network and runs in a disposable project.
When pre-install repository provenance is required, the lead can download the
archive and follow the [documented GitHub attestation verification](REFERENCE.md#conversational-update-notice) before installing; routine installs can use the fast path below.

```sh
curl -fsSL https://raw.githubusercontent.com/veyndrasystems/soulmate/v0.15.0-rc.3/install.sh | sh
export PATH="$HOME/.local/bin:$PATH"
```

Now run the model-free experiment from any directory:

```sh
soulmate benchmark
```

Success starts with `False-completion proof passed (14/14 assertions).` Its
short outcome is:

```text
Attempt 1 failed its current check, so acceptance was refused.
Rework preserved that attempt; a fresh checked attempt still needed review and lead acceptance.
```

The fixture uses real local command exit codes and synthetic actors. It runs in
a disposable Git project, leaves your current project and its worktree
untouched, and removes the temporary project and records by default. To keep
inspectable records, use `soulmate benchmark --output NEW_DIRECTORY`.
This demonstrates the refusal and recovery mechanism, not general token,
time, quality, or adoption outcomes. See the [proof methodology](docs/value-proof-methodology.md).

In checked runs, Soulmate refuses acceptance when the configured check result is missing or reports failure for the current worker artifact. A passing check still
needs fresh review and a lead decision. [Host setup](docs/onboarding.md#ask-your-existing-host-to-manage-the-run)
explains how the existing host carries out a pending assignment.
A real running task may leave a pending review or assignment for your existing host to retrieve.

## Manual setup and the everyday path

From the project directory, initialize portable setup:

```sh
soulmate init --mode portable --root .
```

This writes reviewable configuration to `soulmate.json`, private ignored state
to `.soulmate/`, and generated setup copies for Codex and Claude. It installs no
hooks and does not start agents or grant host permissions. Review the printed
configuration, skill paths, declared scope, and native agent mapping before
project-scoped work; ask before any installation or permission change.

Codex and Claude are generated setup paths. OpenCode is documented as
path-compatible through `.agents/skills/soulmate/`, but host execution is not
exercised by Soulmate CI; OpenCode remains the executor. Other hosts may have
their own documented skill paths and consent rules. [Onboarding details](docs/onboarding.md#keep-the-host-you-already-use).

The normal protocol path is `init -> brief -> run -> check`. A check freezes the
command and binds its result to the submitted artifact. `soulmate check`
validates configuration, profiles, and declared boundaries; it does not run
your project tests. Use the [scripted checked-work example](docs/first-checked-run.md#try-the-complete-example)
for a longer, executable walkthrough.

## When work needs repair or resumption

Ask the agent to resume the run or explain what remains. If you need the manual
record view, `run status` shows the current claim, artifact condition, reported
check, review, and acceptance; `run next` retrieves a validated pending
assignment; `run inspect` retains the history. A failed check needs explicit
rework with fresh artifacts and review.
Recorded artifact bytes must match disk bytes, or no new run event is written.
The record recovers task evidence, not the host's conversation.

## Trust, feedback, and deeper reading

Keep raw ledgers, assignments, and result documents private: they can contain
goals, commands, and paths. Read [SECURITY.md](SECURITY.md) before real work.
The next real-use validation is external users trying the URL-first setup. If
you try it, share what worked, where setup needed clarification, or whether you
reused the record through a [GitHub issue](https://github.com/veyndrasystems/soulmate/issues).

- [First checked run](docs/first-checked-run.md) · [Host setup](docs/onboarding.md)
- [Repair or resume](docs/repair-a-run.md) · [Translate the terminology](docs/glossary.md)
- [Optional memory, hooks, and receipts](docs/optional-surfaces.md)
- [Windows and WSL 2](docs/windows-wsl.md) · [Commands and versioning](REFERENCE.md)
- [Usage findings](docs/participation-validation.md) · [Claim registry](proof/claims.json)
- [Contributing](CONTRIBUTING.md) · [Security](SECURITY.md) · [License](LICENSE)

<details>
<summary>Release and format details</summary>

The public check mapping is deliberately small: v3 supports caller-reported `record-check` only; v4 supports both reported `record-check` and observed `observe-check`. The current `v0.15.0-rc.3` preview creates v4 checked ledgers while retaining readable v3 ledgers; see the [format and reader map](CHANGELOG.md#public-tags-and-format-readers) when choosing a rollback or channel. A check is evidence for the current worker submission, not reviewer approval or lead acceptance.

</details>
