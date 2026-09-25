//! Read-only continuation and applicability projection.

use super::*;

pub(crate) fn has_unresolved(record: &Value) -> bool {
    let c = &record["continuation"];
    if c.is_null() {
        return false;
    }
    if c["source"]["coverageConfirmed"] != true {
        return true;
    }
    let Some(requirements) = c["requirements"].as_array() else {
        return true;
    };
    let Some(supports) = c["supports"].as_array() else {
        return true;
    };
    requirements.iter().any(|req| {
        !supports.iter().any(|support| {
            support["requirementId"] == req["id"]
                && support["requirementRevision"] == req["revision"]
                && support["sourceSha256"] == req["sourceSha256"]
        })
    })
}

pub(crate) fn support_current(loaded: &Loaded, record: &Value) -> Result<bool, String> {
    let c = &record["continuation"];
    if c.is_null() {
        return Ok(true);
    }
    if has_unresolved(record) {
        return Ok(false);
    }
    let ledger = verify_work(
        loaded,
        c["work"].as_str().ok_or("continuation work missing")?,
    )?;
    let (inputs, conditions) = current_conditions(loaded)?;
    for requirement in c["requirements"]
        .as_array()
        .ok_or("requirements malformed")?
    {
        let mut applicable = false;
        for support in c["supports"]
            .as_array()
            .ok_or("supports malformed")?
            .iter()
            .filter(|x| {
                x["requirementId"] == requirement["id"]
                    && x["requirementRevision"] == requirement["revision"]
                    && x["sourceSha256"] == requirement["sourceSha256"]
            })
        {
            if support["inputsSha256"] != inputs || support["conditionsSha256"] != conditions {
                continue;
            }
            if let Some(event) = support["resultEventSha256"].as_str() {
                if run::inspect_event(loaded, &ledger, event).is_ok_and(|found| {
                    found["eventIndex"].as_u64().unwrap_or(0)
                        >= requirement["checkFloorIndex"].as_u64().unwrap_or(0)
                }) {
                    applicable = true;
                    break;
                }
            }
        }
        if !applicable {
            return Ok(false);
        }
    }
    Ok(true)
}

pub(crate) fn continuation_view(loaded: &Loaded, work_id: &str) -> Result<Value, String> {
    let ledger = verify_work(loaded, work_id)?;
    let record = read(&loaded.state_root)?.ok_or("continuation has not been initialized")?;
    if record["goalId"] != work_id || record["continuation"]["work"] != work_id {
        return Err("current goal belongs to another work".into());
    }
    let c = &record["continuation"];
    let (current_inputs, current_conditions) = current_conditions(loaded)?;
    let mut requirements = Vec::new();
    for requirement in c["requirements"]
        .as_array()
        .ok_or("requirements malformed")?
    {
        let supports = c["supports"].as_array().ok_or("supports malformed")?;
        let mut valid = Vec::new();
        for support in supports.iter().filter(|x| {
            x["requirementId"] == requirement["id"]
                && x["requirementRevision"] == requirement["revision"]
                && x["sourceSha256"] == requirement["sourceSha256"]
        }) {
            let event = support["resultEventSha256"].as_str().unwrap_or_default();
            let found = run::inspect_event(loaded, &ledger, event);
            let current = found.is_ok();
            let revision_current = found.is_ok_and(|found| {
                found["eventIndex"].as_u64().unwrap_or(0)
                    >= requirement["checkFloorIndex"].as_u64().unwrap_or(0)
            });
            let applicable = revision_current
                && support["inputsSha256"] == current_inputs
                && support["conditionsSha256"] == current_conditions;
            valid.push(
                json!({"relation":support,"integrityCurrent":current,"revisionCurrent":revision_current,"applicableCurrent":applicable,
                "exactRead":{"command":"run inspect","ledger":ledger,"event":event}}),
            );
        }
        requirements.push(json!({"requirement":requirement,"support":valid,"unresolved":valid.iter().all(|x| x["applicableCurrent"]!=true)}));
    }
    let mut result = json!({"work":work_id,"goalRevision":record["revision"],"source":c["source"],
        "requirements":requirements,"binding":c["binding"],"corrections":c["corrections"],
        "operations":c["operations"],"diagnoses":c["diagnoses"],"children":c["children"],
        "currentInputsSha256":current_inputs,"currentConditionsSha256":current_conditions,
        "wholeGoalReady":false,"readOnly":true});
    result["wholeGoalReady"] = json!(
        c["source"]["coverageConfirmed"] == true
            && result["requirements"]
                .as_array()
                .is_some_and(|xs| xs.iter().all(|x| x["unresolved"] == false))
            && record["closure"]["closed"] == true
    );
    if serde_json::to_vec(&result)
        .map_err(|e| e.to_string())?
        .len()
        > 64 * 1024
    {
        return Err("continuation view exceeds 64 KiB; inspect exact goal record".into());
    }
    Ok(result)
}
