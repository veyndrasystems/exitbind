//! Reaping belongs to an isolated fixture, not the parallel test process.
use super::capture::{process_group_exists, terminate_process_group};
use std::{
    fs,
    os::unix::process::CommandExt,
    process::Command,
    thread,
    time::{Duration, Instant},
};

pub(super) fn external_reap_fixture() {
    #[cfg(target_os = "linux")]
    {
        const HARNESS: &str = "EXITBIND_CLEANUP_REAPER_FIXTURE";
        if std::env::var(HARNESS).as_deref() != Ok("1") {
            let output = Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "run::tests::cleanup_reports_reap_failure_after_still_killing_the_group",
                    "--nocapture",
                ])
                .env(HARNESS, "1")
                .output()
                .expect("launch isolated reaper fixture");
            assert!(
                output.status.success(),
                "isolated reaper fixture failed: {output:?}"
            );
            eprint!("{}", String::from_utf8_lossy(&output.stderr));
            return;
        }
        assert_eq!(
            unsafe { libc::prctl(libc::PR_SET_CHILD_SUBREAPER, 1, 0, 0, 0) },
            0,
            "fixture could not own descendant reaping: {}",
            std::io::Error::last_os_error()
        );
        // Retain the distinction: termination alone leaves a zombie/group until
        // its owner reaps it. The subsequent owned-reaper case keeps all of the
        // original cleanup assertions, including the external leader-reap error.
        run_fixture(false);
    }
    run_fixture(true);
}

fn run_fixture(reap: bool) {
    let marker = std::env::temp_dir().join(format!(
        "exitbind-cleanup-{}-{reap}.ready",
        std::process::id()
    ));
    let mut child = Command::new("sh")
        .args(["-c", "sh -c 'trap : TERM HUP; echo $$ > \"$1\"; while :; do :; done' fixture \"$1\" & exit 0", "fixture"])
        .arg(&marker).process_group(0).spawn().expect("cleanup fixture should launch");
    let group = i32::try_from(child.id()).expect("POSIX fixture group");
    let ready_started = Instant::now();
    while !marker.exists() && ready_started.elapsed() < Duration::from_secs(1) {
        thread::sleep(Duration::from_millis(1));
    }
    let ready = marker.exists();
    let descendant = fs::read_to_string(&marker)
        .expect("descendant readiness")
        .trim()
        .parse::<i32>()
        .expect("descendant pid");
    let waited = unsafe { libc::waitpid(group, std::ptr::null_mut(), 0) };
    let group_before = process_group_exists(group).unwrap_or(false);
    eprintln!("cleanup fixture: reaper={} group={group} descendant={descendant} owned_reaping={reap} before={}", std::process::id(), descendant_state(descendant));
    #[cfg(target_os = "linux")]
    let reaper = reap.then(|| {
        thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(1);
            loop {
                let result = unsafe { libc::waitpid(-group, std::ptr::null_mut(), libc::WNOHANG) };
                if result > 0 {
                    return result;
                }
                if Instant::now() >= deadline {
                    panic!("owned reaper deadline: {}", descendant_state(descendant));
                }
                thread::sleep(Duration::from_millis(1));
            }
        })
    });
    let started = Instant::now();
    let failure =
        terminate_process_group(&mut child).expect_err("external leader reap is an error");
    let elapsed = started.elapsed();
    let group_after = process_group_exists(group);
    let after = descendant_state(descendant);
    eprintln!("cleanup fixture: after={after} group_present={group_after:?} elapsed={elapsed:?} cleanup={failure}");
    #[cfg(target_os = "linux")]
    if let Some(reaper) = reaper {
        assert_eq!(reaper.join().expect("owned reaper result"), descendant);
    } else {
        assert!(
            matches!(group_after, Ok(true)),
            "unreaped group must remain visible: {group_after:?}"
        );
        assert!(
            after.starts_with("zombie-only/unreaped"),
            "termination did not establish zombie-only state: {after}"
        );
        assert!(failure.contains("process group remained"), "{failure}");
        assert_eq!(
            unsafe { libc::waitpid(descendant, std::ptr::null_mut(), 0) },
            descendant
        );
        assert!(
            matches!(process_group_exists(group), Ok(false)),
            "owned final reap did not remove group"
        );
    }
    let _ = fs::remove_file(marker);
    assert_eq!(waited, group);
    assert!(ready, "cleanup fixture did not become ready");
    assert!(
        group_before,
        "cleanup fixture process group was not present"
    );
    if reap {
        assert!(
            matches!(group_after, Ok(false)),
            "cleanup left the process group behind: {group_after:?}; descendant={after}"
        );
    }
    assert!(
        elapsed >= Duration::from_millis(250),
        "cleanup skipped its bounded TERM grace"
    );
    assert!(
        elapsed < Duration::from_secs(1),
        "cleanup exceeded its bounded budget"
    );
    assert!(failure.contains("could not be reaped"), "{failure}");
}

fn descendant_state(pid: i32) -> String {
    #[cfg(target_os = "linux")]
    {
        match fs::read_to_string(format!("/proc/{pid}/stat")) {
            Ok(stat) => {
                let Some((_, fields)) = stat.rsplit_once(") ") else {
                    return "unknown: malformed descendant stat".into();
                };
                let fields: Vec<_> = fields.split_whitespace().collect();
                match fields.as_slice() {
                    [state, parent, group, ..] => format!(
                        "{} pid={pid} parent={parent} group={group}",
                        if *state == "Z" {
                            "zombie-only/unreaped"
                        } else {
                            "live"
                        }
                    ),
                    _ => "unknown: incomplete descendant stat".into(),
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => "fully reaped".into(),
            Err(error) => format!("unknown: descendant state unavailable: {error}"),
        }
    }
    #[cfg(not(target_os = "linux"))]
    format!("unknown: /proc descendant diagnostics unavailable for pid={pid}; operating-system reaper owns orphan")
}
