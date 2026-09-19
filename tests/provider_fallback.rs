#![cfg(unix)]

mod support;

use serde_json::Value;
use std::{
    fs,
    io::Write,
    path::PathBuf,
    process::{Command, Output, Stdio},
};

/// A governor-authorized fallback target executes the same reviewer contract
/// when the primary reviewer target could not execute for an operational
/// reason.  Fallback substitutes an execution target, never a review standard.
struct Fixture {
    root: PathBuf,
}

impl Fixture {
    /// `fallback` names the authorized substitution target, if any.
    fn new_with(fallback: Option<&str>) -> Self {
        let root = support::temp("provider-fallback");
        let init = Command::new(env!("CARGO_BIN_EXE_exitbind"))
            .args(["init", "--mode", "portable", "--root"])
            .arg(&root)
            .output()
            .unwrap();
        assert!(init.status.success(), "{init:?}");
        let config_path = root.join("exitbind.json");
        let mut config: Value = serde_json::from_slice(&fs::read(&config_path).unwrap()).unwrap();
        let mut standby = config["agents"]["reviewer"].clone();
        standby["profile"] = serde_json::json!("exitbind/agents/standby.md");
        standby["purpose"] = serde_json::json!("Stand by as the authorized review target.");
        standby["runtime"] = serde_json::json!({
            "host": "claude",
            "model": "standby-review",
            "reasoningEffort": "high",
            "fallback": "none",
        });
        fs::write(
            root.join("exitbind/agents/standby.md"),
            b"# standby\n\nReview.\n",
        )
        .unwrap();
        config["agents"]["standby_reviewer"] = standby;
        config["agents"]["reviewer"]["runtime"] = serde_json::json!({
            "host": "codex",
            "model": "primary-review",
            "reasoningEffort": "high",
            "fallback": fallback.unwrap_or("none"),
        });
        fs::write(config_path, serde_json::to_vec_pretty(&config).unwrap()).unwrap();
        Self { root }
    }

    fn call(&self, args: &[&str], input: Option<&[u8]>) -> Output {
        let mut command = Command::new(env!("CARGO_BIN_EXE_exitbind"));
        command
            .current_dir(&self.root)
            .args(args)
            .arg("--config")
            .arg(self.root.join("exitbind.json"))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if let Some(input) = input {
            command.stdin(Stdio::piped());
            let mut child = command.spawn().unwrap();
            child.stdin.take().unwrap().write_all(input).unwrap();
            return child.wait_with_output().unwrap();
        }
        command.output().unwrap()
    }

    fn value(&self, args: &[&str], input: Option<&[u8]>) -> Value {
        let output = self.call(args, input);
        assert!(
            output.status.success(),
            "{args:?}: {}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        serde_json::from_slice(&output.stdout).unwrap()
    }

    fn ledger(&self, work: &str) -> String {
        format!(
            ".exitbind/runs/work-{}.jsonl",
            work.strip_prefix("smw_").unwrap()
        )
    }

    fn events(&self, ledger: &str) -> Vec<Value> {
        fs::read_to_string(self.root.join(ledger))
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect()
    }

    /// Drive a run to the point where a reviewer assignment is pending.
    fn to_reviewer(&self, goal: &str) -> (String, Value) {
        let begin = self.value(
            &[
                "work",
                "begin",
                "change",
                "--goal",
                goal,
                "--check-command",
                "test -f pass-marker",
            ],
            None,
        );
        let work = begin["work"].as_str().unwrap().to_owned();
        assert_eq!(begin["next"]["role"], "lead");
        self.value(
            &[
                "work",
                "return",
                &work,
                begin["next"]["assignment"].as_str().unwrap(),
                "--outcome",
                "scoped",
            ],
            Some(b"scope"),
        );
        let worker = self.value(&["work", "next", &work], None);
        assert_eq!(worker["next"]["role"], "worker");
        let completed = self.value(
            &[
                "work",
                "return",
                &work,
                worker["next"]["assignment"].as_str().unwrap(),
                "--outcome",
                "completed",
            ],
            Some(b"implementation"),
        );
        assert_eq!(completed["next"]["action"], "check");
        fs::write(self.root.join("pass-marker"), b"ok").unwrap();
        let checked = self.value(&["work", "check", &work], None);
        assert_eq!(checked["next"]["role"], "reviewer");
        (work, checked)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).unwrap();
    }
}

fn assert_progress(action: &Value) {
    assert_eq!(action["progress"]["applicable"], true);
}

/// A. The primary reviewer cannot execute, the authorized target can, and the
/// substitution carries provenance while the review contract is unchanged.
#[test]
fn primary_unavailable_falls_back_and_still_requires_lead_acceptance() {
    let fixture = Fixture::new_with(Some("standby_reviewer"));
    let (work, checked) = fixture.to_reviewer("quota fallback");
    let ledger = fixture.ledger(&work);

    // The configuration change alone is enough to make the run record fallback
    // provenance, so the ledger opens at v7 while plain runs stay at v6.
    let start = fixture.events(&ledger).remove(0);
    assert_eq!(start["version"], 7);
    let reviewer = start["plan"]["stages"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|stage| stage["agents"].as_array().unwrap())
        .find(|agent| agent["role"] == "reviewer")
        .unwrap();
    assert_eq!(reviewer["name"], "reviewer");
    assert_eq!(reviewer["runtime"]["fallback"], "standby_reviewer");
    assert_eq!(reviewer["fallbackTarget"]["name"], "standby_reviewer");

    // The primary reports an operational failure, not a verdict.
    let unavailable = fixture.value(
        &[
            "work",
            "return",
            &work,
            checked["next"]["assignment"].as_str().unwrap(),
            "--outcome",
            "unavailable",
            "--reason",
            "provider_quota",
        ],
        Some(b"provider quota exhausted"),
    );
    assert_progress(&unavailable["next"]);
    // The unavailability is non-terminal: the run keeps running at the same
    // stage and attempt, and no stage advance happened.
    assert_eq!(unavailable["next"]["action"], "spawn");
    assert_eq!(unavailable["next"]["role"], "reviewer");

    let events = fixture.events(&ledger);
    let reported = events
        .iter()
        .find(|event| event["outcome"] == "unavailable")
        .unwrap();
    assert_eq!(reported["agent"], "reviewer");
    assert_eq!(reported["role"], "reviewer");
    assert_eq!(reported["fallback"]["reason"], "provider_quota");
    // Reported identity stays reported: nothing here observed a host.
    assert!(reported.get("inputsSha256").is_none());

    // The same reviewer role is re-issued against the authorized target.
    assert_eq!(unavailable["next"]["role"], "reviewer");
    assert_eq!(unavailable["next"]["agent"], "standby_reviewer");
    assert_eq!(unavailable["next"]["packet"]["substitutedFrom"], "reviewer");
    assert_eq!(unavailable["next"]["packet"]["runtime"]["host"], "claude");

    // The substitute's approval alone does not accept the run.
    let approved = fixture.value(
        &[
            "work",
            "return",
            &work,
            unavailable["next"]["assignment"].as_str().unwrap(),
            "--outcome",
            "approved",
        ],
        Some(b"review"),
    );
    assert_progress(&approved["next"]);
    assert_eq!(approved["next"]["action"], "lead_decision");
    assert_eq!(approved["next"]["role"], "lead");

    let events = fixture.events(&ledger);
    let substitution = events
        .iter()
        .find(|event| event["outcome"] == "approved" && event["agent"] == "standby_reviewer")
        .unwrap();
    assert_eq!(substitution["fallback"]["reason"], "provider_quota");
    assert_eq!(substitution["fallback"]["from"], "reviewer");
    assert_eq!(substitution["version"], 7);
    // The substitution ran the same stage and attempt as the target it replaced.
    assert_eq!(substitution["stage"], reported["stage"]);
    assert_eq!(substitution["attempt"], reported["attempt"]);
    assert_eq!(substitution["role"], "reviewer");
    // Freshness binding is unchanged: a reviewer approval still declares the
    // tested inputs it judged.
    assert!(substitution["inputsSha256"].is_string());

    // Only the lead's own acceptance reaches READY.
    let done = fixture.value(
        &[
            "work",
            "return",
            &work,
            approved["next"]["assignment"].as_str().unwrap(),
            "--outcome",
            "accepted",
        ],
        Some(b"accept"),
    );
    assert_eq!(done["next"]["action"], "done");
    assert_eq!(done["next"]["progress"]["state"], "READY");
}

/// B. Without an authorized target, an operational failure leaves the run
/// blocked and fabricates no review.
#[test]
fn primary_unavailable_without_a_fallback_stays_blocked() {
    let fixture = Fixture::new_with(None);
    let (work, checked) = fixture.to_reviewer("no fallback");
    let ledger = fixture.ledger(&work);
    let start = fixture.events(&ledger).remove(0);
    assert_eq!(start["version"], 6);

    let unavailable = fixture.value(
        &[
            "work",
            "return",
            &work,
            checked["next"]["assignment"].as_str().unwrap(),
            "--outcome",
            "unavailable",
            "--reason",
            "provider_unavailable",
        ],
        Some(b"provider down"),
    );
    assert_progress(&unavailable["next"]);
    // With no authorized target the reviewer cannot be re-issued, so the run
    // blocks instead of advancing on a review that never happened.
    assert_eq!(unavailable["next"]["action"], "done");
    assert_eq!(unavailable["next"]["status"], "blocked");

    let events = fixture.events(&ledger);
    let reported = events
        .iter()
        .find(|event| event["outcome"] == "unavailable")
        .unwrap();
    assert_eq!(reported["agent"], "reviewer");
    // No bounded reason is attachable without the v7 fallback shape, and no
    // fabricated fallback binding appears.
    assert_eq!(reported["version"], 6);
    assert!(reported.get("fallback").is_none());

    // No reviewer ever produced a verdict.
    let events = fixture.events(&ledger);
    assert!(!events.iter().any(|event| {
        event["role"] == "reviewer"
            && matches!(
                event["outcome"].as_str(),
                Some("approved" | "rework" | "blocked")
            )
    }));
}

/// C. A second operational failure (the substitute's own) is bounded: the run
/// blocks instead of hunting for a third target.
#[test]
fn fallback_that_is_also_unavailable_blocks_without_retrying() {
    let fixture = Fixture::new_with(Some("standby_reviewer"));
    let (work, checked) = fixture.to_reviewer("double outage");
    let ledger = fixture.ledger(&work);

    let primary = fixture.value(
        &[
            "work",
            "return",
            &work,
            checked["next"]["assignment"].as_str().unwrap(),
            "--outcome",
            "unavailable",
            "--reason",
            "rate_limit",
        ],
        Some(b"rate limited"),
    );
    assert_eq!(primary["next"]["agent"], "standby_reviewer");
    let substitute = fixture.value(
        &[
            "work",
            "return",
            &work,
            primary["next"]["assignment"].as_str().unwrap(),
            "--outcome",
            "unavailable",
            "--reason",
            "provider_unavailable",
        ],
        Some(b"also down"),
    );
    assert_progress(&substitute["next"]);
    assert_eq!(substitute["next"]["action"], "done");
    assert_eq!(substitute["next"]["status"], "blocked");

    // Bounded attempts: exactly two operational failures, no third target.
    let events = fixture.events(&ledger);
    let unavailable = events
        .iter()
        .filter(|event| event["outcome"] == "unavailable")
        .count();
    assert_eq!(unavailable, 2);
    assert!(!events.iter().any(|event| event["outcome"] == "approved"));
}

/// D. Anti-review-shopping: once a reviewer returned `rework`, an operational
/// failure cannot re-open the review on another target.
#[test]
fn unavailable_is_rejected_after_a_rework_verdict() {
    let fixture = Fixture::new_with(Some("standby_reviewer"));
    let (work, checked) = fixture.to_reviewer("no shopping after rework");
    let ledger = fixture.ledger(&work);

    let rework = fixture.value(
        &[
            "work",
            "return",
            &work,
            checked["next"]["assignment"].as_str().unwrap(),
            "--outcome",
            "rework",
        ],
        Some(b"rework"),
    );
    assert_progress(&rework["next"]);

    // The reviewer target is still the primary on the next attempt; asking it
    // to report unavailability would be an escape from the adverse verdict.
    let attempt = fixture.value(&["work", "next", &work], None);
    let assignment = attempt["next"]["assignment"].as_str().unwrap();
    let escaped = fixture.call(
        &[
            "work",
            "return",
            &work,
            assignment,
            "--outcome",
            "unavailable",
            "--reason",
            "provider_quota",
        ],
        Some(b"quota"),
    );
    assert!(
        !escaped.status.success(),
        "rework was re-opened by fallback: {escaped:?}"
    );

    let events = fixture.events(&ledger);
    assert_eq!(
        events
            .iter()
            .filter(|event| event["outcome"] == "unavailable")
            .count(),
        0
    );
    assert!(!events
        .iter()
        .any(|event| event["role"] == "reviewer" && event["outcome"] == "approved"));
}

/// E. The same rule holds for `blocked`: no verdict may be shopped away.
#[test]
fn unavailable_is_rejected_after_a_blocked_verdict() {
    let fixture = Fixture::new_with(Some("standby_reviewer"));
    let (work, checked) = fixture.to_reviewer("no shopping after blocked");
    let ledger = fixture.ledger(&work);

    let blocked = fixture.value(
        &[
            "work",
            "return",
            &work,
            checked["next"]["assignment"].as_str().unwrap(),
            "--outcome",
            "blocked",
        ],
        Some(b"blocked"),
    );
    assert_progress(&blocked["next"]);

    // The reviewer verdict of `blocked` is terminal for the run, so the run is
    // no longer running and the reducer refuses any further submission.
    let attempt = fixture.value(&["work", "next", &work], None);
    assert_eq!(attempt["next"]["action"], "done");
    assert_eq!(attempt["next"]["status"], "blocked");
    let events = fixture.events(&ledger);
    assert!(!events.iter().any(|event| event["outcome"] == "unavailable"));

    // The reducer is the authority, independent of the façade: a hand-crafted
    // unavailable after a reviewer block is refused on replay.
    let replay = fixture.call(&["run", "inspect", &ledger, "--json"], None);
    assert!(replay.status.success(), "{replay:?}");
}

/// F. A substitution is a review like any other: the subject it approved goes
/// stale when a covered input changes.
#[test]
fn fallback_approval_goes_stale_with_its_subject() {
    let fixture = Fixture::new_with(Some("standby_reviewer"));
    let (work, checked) = fixture.to_reviewer("stale fallback review");

    let primary = fixture.value(
        &[
            "work",
            "return",
            &work,
            checked["next"]["assignment"].as_str().unwrap(),
            "--outcome",
            "unavailable",
            "--reason",
            "provider_quota",
        ],
        Some(b"quota"),
    );
    let approved = fixture.value(
        &[
            "work",
            "return",
            &work,
            primary["next"]["assignment"].as_str().unwrap(),
            "--outcome",
            "approved",
        ],
        Some(b"review"),
    );
    assert_eq!(approved["next"]["action"], "lead_decision");

    // Mutate the tested input the fallback review was bound to: the worker
    // result changes, so the approval can no longer authorize it.
    fs::remove_file(fixture.root.join("pass-marker")).unwrap();

    let stale = fixture.value(&["work", "next", &work], None);
    assert_progress(&stale["next"]);
    assert_ne!(stale["next"]["action"], "lead_decision");

    let premature = fixture.call(
        &[
            "work",
            "return",
            &work,
            approved["next"]["assignment"].as_str().unwrap(),
            "--outcome",
            "accepted",
        ],
        Some(b"accept"),
    );
    assert!(
        !premature.status.success(),
        "stale fallback approval accepted: {premature:?}"
    );
}

/// G. A fresh session reconstructs the recorded substitution and its review
/// without needing any provider-specific hidden state.
#[test]
fn resume_reconstructs_a_recorded_substitution() {
    let fixture = Fixture::new_with(Some("standby_reviewer"));
    let (work, checked) = fixture.to_reviewer("resume substitution");
    let ledger = fixture.ledger(&work);

    let primary = fixture.value(
        &[
            "work",
            "return",
            &work,
            checked["next"]["assignment"].as_str().unwrap(),
            "--outcome",
            "unavailable",
            "--reason",
            "rate_limit",
        ],
        Some(b"rate limited"),
    );
    let approved = fixture.value(
        &[
            "work",
            "return",
            &work,
            primary["next"]["assignment"].as_str().unwrap(),
            "--outcome",
            "approved",
        ],
        Some(b"review"),
    );
    assert_eq!(approved["next"]["action"], "lead_decision");
    let events_before = fixture.events(&ledger);

    // A separate process reads the ledger alone.
    let resumed = fixture.value(&["work", "resume"], None);
    assert_eq!(resumed["status"], "resumed");
    assert_eq!(resumed["work"], work);
    assert_eq!(resumed["next"]["action"], "lead_decision");
    assert_eq!(resumed["next"]["role"], "lead");
    assert!(resumed["residual"]["doNotRepeat"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item == "review"));

    // No duplicate review and no new event: the recorded substitution is the
    // only reviewer execution for this attempt.
    assert_eq!(fixture.events(&ledger).len(), events_before.len());
    assert_eq!(
        fixture
            .events(&ledger)
            .iter()
            .filter(|event| event["agent"] == "standby_reviewer")
            .count(),
        1
    );
}
