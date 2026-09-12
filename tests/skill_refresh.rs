mod support;
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Output},
};

const SOULMATE_SKILL: &[u8] = include_bytes!("../skills/soulmate/SKILL.md");
const AWAY_GUIDE: &str = include_str!("../docs/codex-tmux-away.md");
const REFERENCE: &str = include_str!("../REFERENCE.md");
const EXTERNAL_SENTINEL: &[u8] = b"host-owned sentinel\n";

fn temp(label: &str) -> PathBuf {
    support::temp(&format!("skill-refresh-{label}"))
}

fn invoke(arguments: &[&str], bindings: &Path) -> Output {
    let home = bindings.parent().unwrap().join("home");
    fs::create_dir_all(&home).unwrap();
    Command::new(env!("CARGO_BIN_EXE_soulmate"))
        .env("HOME", home)
        .env("SOULMATE_BINDINGS_DIR", bindings)
        .args(arguments)
        .output()
        .unwrap()
}

fn output_text(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn local_project(label: &str) -> (PathBuf, PathBuf, PathBuf, PathBuf) {
    let base = temp(label);
    fs::write(base.join("external-sentinel"), EXTERNAL_SENTINEL).unwrap();
    let product = base.join("product");
    let control = base.join("control");
    let state = base.join("state");
    let bindings = base.join("bindings");
    for path in [&product, &control, &state, &bindings] {
        fs::create_dir(path).unwrap();
    }
    let initialized = invoke(
        &[
            "init",
            "--mode",
            "local",
            "--project-id",
            "skill_refresh_fixture",
            "--root",
            product.to_str().unwrap(),
            "--control-root",
            control.to_str().unwrap(),
            "--state-root",
            state.to_str().unwrap(),
        ],
        &bindings,
    );
    assert!(
        initialized.status.success(),
        "{}",
        output_text(&initialized)
    );
    (base, product, control, bindings)
}

#[test]
fn attended_work_uses_native_spawn_without_away_fallback() {
    let (base, _product, control, _bindings) = local_project("native-spawn");
    let agents_projection =
        fs::read_to_string(control.join(".agents/skills/soulmate/SKILL.md")).unwrap();
    let claude_projection =
        fs::read_to_string(control.join(".claude/skills/soulmate/SKILL.md")).unwrap();
    let source = std::str::from_utf8(SOULMATE_SKILL).unwrap();
    assert_eq!(agents_projection.as_bytes(), SOULMATE_SKILL);
    assert_eq!(claude_projection.as_bytes(), SOULMATE_SKILL);
    let normalize = |text: &str| text.split_whitespace().collect::<Vec<_>>().join(" ");

    for skill in [source, &agents_projection, &claude_projection] {
        let skill = normalize(skill);
        assert!(skill.contains("every implementation worker and reviewer must use the host's native subagent spawn with the assignment's exact `nativeTaskName`"));
        assert!(skill.contains("native spawn is unavailable, stop and return the pending assignment to the operator; do not fall back to shell `codex exec` or `soulmate away`"));
        assert!(skill.contains("`openai/codex#31894`"));
        assert!(skill.contains(
            "`soulmate away` remains reserved for an explicit operator-away/disconnect handoff"
        ));
        assert!(skill.contains("soulmate away start AGENT LEDGER"));
        assert!(skill.contains(
            "Before `run start`, the coordinating root must select a fresh, unambiguous ledger path beneath configured StateRoot"
        ));
        assert!(skill.contains("never use a basename-only ProductRoot path"));
        assert!(skill
            .contains("Retain and reuse that exact path through status, recovery, and reporting"));
        assert!(skill
            .contains("do not ask the human to transport or decide routine ledger bookkeeping"));
    }

    for document in [AWAY_GUIDE, REFERENCE] {
        let document = normalize(document);
        assert!(document.contains("openai/codex#31894"));
        assert!(document.contains("a strong external symptom match"));
        assert!(document.contains("not a proven root cause"));
        assert!(document.contains("exclude affected `codex exec` no-result samples from provider-native completion and token-efficiency baselines"));
        assert!(document.contains(
            "Historical evidence remains in place; quarantine does not delete or rewrite it"
        ));
        assert!(document.contains("soulmate away start implementation_worker"));
    }

    fs::remove_dir_all(base).unwrap();
}

#[test]
fn portable_init_and_refresh_distribute_native_continuity_guidance() {
    let base = temp("portable-continuity");
    let product = base.join("product");
    let bindings = base.join("bindings");
    fs::create_dir(&product).unwrap();
    let initialized = invoke(
        &[
            "init",
            "--mode",
            "portable",
            "--root",
            product.to_str().unwrap(),
        ],
        &bindings,
    );
    assert!(
        initialized.status.success(),
        "{}",
        output_text(&initialized)
    );
    let source = std::str::from_utf8(SOULMATE_SKILL).unwrap();
    assert!(source.contains("## Native conversation continuity"));
    let config_before = fs::read(product.join("soulmate.json")).unwrap();
    let paths = [
        product.join(".agents/skills/soulmate/SKILL.md"),
        product.join(".claude/skills/soulmate/SKILL.md"),
    ];
    for path in &paths {
        assert_eq!(fs::read(path).unwrap(), SOULMATE_SKILL);
        fs::write(
            path,
            "<!-- soulmate-managed-skill:v1 -->\nOld managed skill.\n",
        )
        .unwrap();
    }
    let refreshed = invoke(
        &[
            "init",
            "--refresh-skills",
            "--root",
            product.to_str().unwrap(),
        ],
        &bindings,
    );
    assert!(refreshed.status.success(), "{}", output_text(&refreshed));
    for path in paths {
        assert_eq!(fs::read(path).unwrap(), SOULMATE_SKILL);
    }
    assert_eq!(
        fs::read(product.join("soulmate.json")).unwrap(),
        config_before
    );
    fs::remove_dir_all(base).unwrap();
}

#[test]
fn refresh_restores_missing_skills_and_reports_each_state() {
    let (base, product, control, bindings) = local_project("states");
    let product_before = fs::read_dir(&product).unwrap().count();
    let selected = [
        (
            control.join(".agents/skills/soulmate/SKILL.md"),
            SOULMATE_SKILL,
        ),
        (
            control.join(".claude/skills/soulmate/SKILL.md"),
            SOULMATE_SKILL,
        ),
    ];
    for (path, _) in &selected {
        fs::remove_file(path).unwrap();
    }

    let first = invoke(
        &[
            "init",
            "--refresh-skills",
            "--root",
            control.to_str().unwrap(),
        ],
        &bindings,
    );
    assert!(first.status.success(), "{}", output_text(&first));
    let first_text = output_text(&first);
    for (path, expected) in &selected {
        assert_eq!(fs::read(path).unwrap(), *expected);
        assert!(first_text.contains(&format!("created {}", relative_skill(path, &control))));
    }
    assert_eq!(fs::read_dir(&product).unwrap().count(), product_before);

    let second = invoke(
        &[
            "init",
            "--refresh-skills",
            "--root",
            control.to_str().unwrap(),
        ],
        &bindings,
    );
    assert!(second.status.success(), "{}", output_text(&second));
    let second_text = output_text(&second);
    for (path, _) in &selected {
        assert!(second_text.contains(&format!("unchanged {}", relative_skill(path, &control))));
    }

    fs::OpenOptions::new()
        .append(true)
        .open(&selected[0].0)
        .unwrap()
        .write_all(b"\n")
        .unwrap();
    let third = invoke(
        &[
            "init",
            "--refresh-skills",
            "--root",
            control.to_str().unwrap(),
        ],
        &bindings,
    );
    assert!(third.status.success(), "{}", output_text(&third));
    assert!(output_text(&third).contains(&format!(
        "refreshed {}",
        relative_skill(&selected[0].0, &control)
    )));
    assert_eq!(fs::read(&selected[0].0).unwrap(), SOULMATE_SKILL);

    fs::remove_dir_all(base).unwrap();
}

#[test]
fn refresh_conflict_preflight_prevents_earlier_missing_creation() {
    let (base, _product, control, bindings) = local_project("conflict");
    let missing = control.join(".agents/skills/soulmate/SKILL.md");
    let conflict = control.join(".claude/skills/soulmate/SKILL.md");
    fs::remove_file(&missing).unwrap();
    fs::write(&conflict, "operator-owned skill\n").unwrap();

    let refused = invoke(
        &[
            "init",
            "--refresh-skills",
            "--root",
            control.to_str().unwrap(),
        ],
        &bindings,
    );
    assert!(!refused.status.success());
    assert!(output_text(&refused).contains("refusing to overwrite existing project skill"));
    assert!(!missing.exists());
    assert_eq!(
        fs::read_to_string(conflict).unwrap(),
        "operator-owned skill\n"
    );

    fs::remove_dir_all(base).unwrap();
}

fn relative_skill(path: &Path, control: &Path) -> String {
    path.strip_prefix(control)
        .unwrap()
        .to_string_lossy()
        .into_owned()
}
#[test]
fn reality_and_decision_guidance_is_embedded_and_distributed() {
    let (base, _product, control, _bindings) = local_project("reality-decision");
    let source = std::str::from_utf8(SOULMATE_SKILL).unwrap();
    let projections = [
        control.join(".agents/skills/soulmate/SKILL.md"),
        control.join(".claude/skills/soulmate/SKILL.md"),
    ];
    assert_eq!(
        fs::read(base.join("external-sentinel")).unwrap(),
        EXTERNAL_SENTINEL
    );
    let normalize = |text: &str| text.split_whitespace().collect::<Vec<_>>().join(" ");
    let required = [
        "authored source -> projection -> discovery -> invocation -> effective instructions/permissions -> behavior -> outcome",
        "independently supported endpoint facts",
        "minimum bridge assumption",
        "test that connecting edge directly",
        "endpoint presence or success does not prove the next link",
        "verified within scope",
        "failed",
        "unverified",
        "strongest supported claim",
        "smallest decisive next probe",
        "installation shows presence",
        "none of these alone proves invocation",
        "absent observed need or advantage in the tested context",
        "faulty implementation",
        "inconclusive test",
        "change",
        "keep",
        "defer",
        "stop",
        "remaining unknowns",
        "reopening condition",
        "does not require initialization, a ledger, extra agents, durable memory, or a governed run",
        "ordinary single-agent work proceeds directly",
        "a standing project preference makes soulmate available for eligible work",
        "does not govern every task",
        "independent review, resumability, or deterministic handoff is actually needed",
        "routine small, reversible edits stay direct",
        "exact run, attempt, artifact, and context",
        "may coexist across different findings",
        "unavailable observation remains unverified",
        "insufficient evidence is not product validation",
        "precise owner decision",
        "before a materially costly dependent phase, or before replacing or retiring a working capability",
        "service, execution environment or host, and the relevant configuration, session, or capability",
        "cheapest decisive, authorized evidence already available",
        "unknown, merely reported, stale, or adjacent evidence",
        "distinguish early feasibility from replacement readiness",
        "actual supported-entry, selection, invocation, and required-behavior checks before cutover",
        "a blocker stops only dependent work",
        "recheck only required conditions that are volatile or were invalidated",
        "do not retire a working fallback while any required replacement target lacks fresh target-matching evidence",
        "a local native child is not execution on a remote target",
        "keep ordinary reversible single-host work direct and proportionate",
    ];
    for skill in std::iter::once(source.to_owned()).chain(
        projections
            .iter()
            .map(|path| fs::read_to_string(path).unwrap()),
    ) {
        let normalized = normalize(&skill).to_lowercase();
        for phrase in required {
            assert!(
                normalized.contains(phrase),
                "missing {phrase:?} in distributed Soulmate skill"
            );
        }
        let readiness = normalized.find("## target-bound readiness").unwrap();
        let continuity = normalized
            .find("## native conversation continuity")
            .unwrap();
        assert!(
            readiness < continuity,
            "readiness guidance moved after continuity"
        );
        let ordering = [
            "before a materially costly dependent phase",
            "distinguish early feasibility from replacement readiness",
            "a blocker stops only dependent work",
            "immediately before cutover",
            "a local native child is not execution on a remote target",
            "keep ordinary reversible single-host work direct and proportionate",
        ];
        for pair in ordering.windows(2) {
            assert!(
                normalized.find(pair[0]).unwrap() < normalized.find(pair[1]).unwrap(),
                "readiness guidance order changed: {:?}",
                pair
            );
        }
    }
    for projection in projections {
        assert_eq!(fs::read(projection).unwrap(), SOULMATE_SKILL);
    }
    fs::remove_dir_all(base).unwrap();
}
