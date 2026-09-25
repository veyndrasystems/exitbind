# Continue one work item across Codex and Claude Code

Exitbind keeps the portable work identity in the existing run and canonical
session-goal histories. Native sessions and children remain in their own hosts.
This local-workspace path assumes the same authorized operating-system
principal on both hosts; a host name supplied to the CLI is an observation, not
an authentication credential.

Start from an explicit `smw_…` work locator. `exitbind work next WORK` includes a
bounded `continuation` view when this work has been initialized. If that field
says `requiresExpansion`, run `exitbind work continuation WORK` before acting.
The latter is read-only and includes the source requirements, current lead
binding, corrections, result references, operations, diagnoses, children, and
remaining requirements. It never launches or retries an operation. `work
resume` includes the same view only when discovery finds exactly one current
work item. An ambiguous resume still requires an explicit locator.

The coordinating lead initializes the continuation once by piping a JSON
object to `exitbind work record WORK`. `sourceText` retains the original
request; every initial requirement must be a unique, exact excerpt of it.
Unrepresented source clauses remain a coverage question until a lead records
an explicit `cover` assertion against the source hash. That assertion is
`lead_asserted_review_required`, not semantic proof by itself.

```json
{
  "action": "init",
  "sourceRef": "requirements.md",
  "sourceText": "First requirement. Second requirement.",
  "requirements": [
    {"id": "first", "text": "First requirement."},
    {"id": "second", "text": "Second requirement."}
  ]
}
```

`bind` records one current lead host/session and increments its binding
revision. It requires the current goal revision and the previous binding
revision. Repeating the identical bind has no effect; a stale or conflicting
bind is refused. Subsequent records require both `expectedRevision` and
`bindingRevision`. A `refine` increments only the named requirement revision
and retains its prior text. Existing support remains recorded but no longer
applies to the new revision.

`support` accepts only a passing observed check event from this exact work
whose recorded `requirementId` matches the named requirement. It also binds
the requirement revision, source hash, tested inputs, configuration and
conditions hash. A refinement records the current check-event floor, so an
older event cannot be attached to the new requirement revision. This floor
orders events in the selected single-workspace path; it is not an atomic
transaction across concurrent goal and run writers. `work continuation`
rechecks the event read-only and reports
integrity separately from current applicability. Its `exactRead` locator uses
`run inspect LEDGER --event EVENT`; reading it does not execute the check or
resubmit a held result. A passing check for another requirement cannot close
this one, and whole-goal readiness still requires explicit lead closure.

`correct` records an operator-asserted scoped correction and an optional
same-scope supersession. The CLI cannot independently prove the caller's human
authority. `intent` records an operation before dispatch with a stable request
ID, parameter digest and origin-qualified binding. Its initial status is
`uncertain`; duplicate identical intent is harmless, while conflicting
parameters are refused. `observe` records a status or native handle from the
origin host. No missing process-list entry authorizes another dispatch.
`diagnose` records the observation class, conditions and invalidation rule.

`child` accepts the exact text returned through a native host child tool,
with a digest, assignment and host-qualified child handle. The result is
marked `host_reported_native_child`: transporting bytes does not make it an
independent review or a lead acceptance. The CLI accepts up to 8 KiB of child
text in a 16 KiB input object; larger outputs need a separately assessed
artifact path and cannot be silently shortened. The view is bounded at 64 KiB.
Only the relevant local task state should cross hosts; transcripts, hidden
reasoning, credentials and unrelated personal memory do not belong in these
records.

This path does not migrate a live process between hosts, authenticate native
session claims, control external shell effects, or replace each host's own
permission decisions. When the destination cannot perform an action, keep the
obligation and report the limitation instead of broadening permissions.
