//! Pending assignment packets derived from validated run state.

use serde_json::{json, Value};
use std::collections::BTreeSet;

pub(crate) fn pending(state: &Value) -> Vec<Value> {
    if state["status"] != "running" {
        return Vec::new();
    }
    let stage_num = state["currentStage"].as_u64().unwrap_or(0);
    let stage = state["plan"]["stages"]
        .as_array()
        .and_then(|stages| stages.iter().find(|stage| stage["stage"] == stage_num));
    let Some(stage) = stage else {
        return Vec::new();
    };
    let Some(submissions) = state["submissions"].as_array() else {
        return Vec::new();
    };
    let Some(agents) = stage["agents"].as_array() else {
        return Vec::new();
    };
    let submitted: BTreeSet<&str> = submissions
        .iter()
        .filter(|event| {
            event["stage"] == state["currentStage"] && event["attempt"] == state["attempt"]
        })
        .filter_map(|event| event["agent"].as_str())
        .collect();
    let upstream: Vec<Value> = submissions
        .iter()
        .filter(|event| {
            let attempt = event["attempt"].as_u64().unwrap_or(0);
            let current_attempt = state["attempt"].as_u64().unwrap_or(0);
            let stage = event["stage"].as_u64().unwrap_or(0);
            let current_stage = state["currentStage"].as_u64().unwrap_or(0);
            attempt < current_attempt || (attempt == current_attempt && stage < current_stage)
        })
        .map(|event| json!({
            "stage": event["stage"],
            "attempt": event["attempt"],
            "agent": event["agent"],
            "role": event["role"],
            "root": event["artifact"]["root"].as_str().unwrap_or("product"),
            "path": event["artifact"]["path"],
            "sha256": event["artifact"]["sha256"],
            "attemptStatus": if event["attempt"] == state["attempt"] { "current" } else { "prior" }
        }))
        .collect();
    let limit = state["plan"]["maxParallel"]
        .as_u64()
        .unwrap_or(agents.len() as u64)
        .max(1) as usize;
    let mut assignments: Vec<Value> = agents
        .iter()
        .filter(|agent| !submitted.contains(agent["name"].as_str().unwrap_or("")))
        .map(|agent| packet(state, agent, &upstream))
        .take(limit)
        .collect();
    // A reviewer target that could not execute is re-issued against its
    // authorized fallback target: same stage, attempt, and role — only the
    // execution target is substituted.  At most one substitution per
    // stage+attempt is ever issued.
    for agent in agents {
        let Some(target) = agent
            .get("fallbackTarget")
            .filter(|target| target.is_object())
        else {
            continue;
        };
        let name = agent["name"].as_str().unwrap_or("");
        let unavailable = submissions.iter().filter(|event| {
            event["stage"] == state["currentStage"]
                && event["attempt"] == state["attempt"]
                && event["agent"] == name
                && event["outcome"] == "unavailable"
        });
        let Some(primary) = unavailable.last() else {
            continue;
        };
        let target_name = target["name"].as_str().unwrap_or("");
        let target_submitted = submissions.iter().any(|event| {
            event["stage"] == state["currentStage"]
                && event["attempt"] == state["attempt"]
                && event["agent"] == target_name
        });
        if target_submitted || submitted.contains(target_name) {
            continue;
        }
        assignments.push(substitution(state, agent, target, primary, &upstream));
    }
    assignments
}

/// The substitution packet: the fallback target's own runtime and profile, with
/// the primary's operational reason carried so the substitution is legible
/// without upgrading caller-reported identity into observed evidence.
fn substitution(
    state: &Value,
    primary: &Value,
    target: &Value,
    unavailable: &Value,
    upstream: &[Value],
) -> Value {
    let mut packet = packet(state, target, upstream);
    packet["substitutedFrom"] = primary["name"].clone();
    packet["substitutionReason"] = unavailable["fallback"]["reason"].clone();
    // Marks this packet as already-substituted so a second operational failure
    // is bounded to blocked rather than opening another target.
    packet["fallbackTarget"] = json!({
        "name": target["name"],
        "substitutedFrom": primary["name"],
    });
    if let Some(display) = primary.get("displayName") {
        packet["substitutedFromDisplayName"] = display.clone();
    }
    packet
}

fn packet(state: &Value, agent: &Value, upstream: &[Value]) -> Value {
    let mut assignment = json!({
        "stage": state["currentStage"],
        "attempt": state["attempt"],
        "agent": agent["name"],
        "displayName": agent["displayName"],
        "nativeTaskName": agent["nativeTaskName"],
        "role": agent["role"],
        "goal": state["goal"],
        "purpose": agent["purpose"],
        "profile": {"path": agent["profile"], "sha256": agent["profileSha256"]},
        "profilePath": agent["profile"],
        "profileSha256": agent["profileSha256"],
        "runtime": agent["runtime"],
        "declaredBoundary": agent["declaredBoundary"],
        "upstreamArtifacts": upstream
    });
    let run_short = state["runId"]
        .as_str()
        .unwrap_or("run")
        .chars()
        .take(12)
        .collect::<String>();
    assignment["artifactRootHint"] = json!("state");
    let state_namespace = crate::project_layout::state_namespace();
    assignment["artifactPathHint"] = json!(format!(
        "{state_namespace}/artifacts/{run_short}-{}-stage-{}-attempt-{}.md",
        agent["name"].as_str().unwrap_or("agent"),
        state["currentStage"],
        state["attempt"]
    ));
    assignment["upstreamArtifactsImmutable"] = json!(true);
    if let Some(references) = agent.get("memoryReferences") {
        assignment["memoryReferences"] = references.clone();
    }
    if let Some(receipt) = state.get("harnessReceipt") {
        assignment["harnessReceipt"] = receipt.clone();
    }
    if let Some(policy) = state.get("checkPolicy") {
        assignment["checkPolicy"] = policy.clone();
    }
    if let Some(target) = agent
        .get("fallbackTarget")
        .filter(|target| target.is_object())
    {
        // The authorized substitution target travels with the primary packet so
        // a standby target's identity is never guessed at execution time.
        assignment["fallbackTarget"] = target.clone();
    }
    assignment
}
