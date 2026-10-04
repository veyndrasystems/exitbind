//! Bootstrap keeps source identity while authority and runtime state are owned
//! outside a read-only source. No source relocation or writable .git is needed.
#![cfg(unix)]
mod support;
use serde_json::Value;
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Command, Output},
};

struct Fixture {
    base: PathBuf,
    source: PathBuf,
    state: PathBuf,
    bindings: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let base = support::temp("portability");
        let source = base.join("source");
        let state = base.join("user-state");
        let bindings = state.join("bindings");
        for path in [&source, &state] {
            fs::create_dir(path).unwrap();
        }
        fs::create_dir(source.join("nested")).unwrap();
        Self {
            base,
            source,
            state,
            bindings,
        }
    }
    fn call(&self, cwd: &Path, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_exitbind"))
            .current_dir(cwd)
            .args(args)
            .env("XDG_STATE_HOME", &self.state)
            .env("EXITBIND_BINDINGS_DIR", &self.bindings)
            .env("HOME", &self.base)
            .output()
            .unwrap()
    }
    fn context(&self, cwd: &Path) -> Value {
        let out = self.call(cwd, &["project", "context", "--json"]);
        assert!(out.status.success(), "{out:?}");
        serde_json::from_slice(&out.stdout).unwrap()
    }
    fn git(&self, args: &[&str]) {
        assert!(Command::new("git")
            .arg("-C")
            .arg(&self.source)
            .args(args)
            .status()
            .unwrap()
            .success());
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        if std::thread::panicking() {
            eprintln!("retained portability fixture: {}", self.base.display());
            return;
        }
        for root in [&self.base, &self.state, &self.bindings] {
            if root.exists() {
                fs::set_permissions(root, fs::Permissions::from_mode(0o700)).unwrap();
            }
        }
        fs::set_permissions(&self.source, fs::Permissions::from_mode(0o700)).unwrap();
        fs::remove_dir_all(&self.base).unwrap();
    }
}

#[test]
fn readonly_source_bootstraps_without_surgery_and_nested_context_has_one_identity() {
    let f = Fixture::new();
    fs::write(f.source.join("keep.txt"), "source sentinel").unwrap();
    fs::set_permissions(&f.source, fs::Permissions::from_mode(0o555)).unwrap();
    let out = f.call(&f.source, &["init", "--skip-skills"]);
    assert!(out.status.success(), "{out:?}");
    assert!(!f.source.join("exitbind.json").exists());
    let root = f.context(&f.source);
    let nested = f.context(&f.source.join("nested"));
    assert_eq!(
        root["project"]["memoryIdentity"],
        nested["project"]["memoryIdentity"]
    );
    assert_eq!(root["placement"]["sourceRoot"], f.source.to_str().unwrap());
    assert_eq!(root["placement"]["sourceCapability"], "read-only");
    assert_eq!(root["placement"]["stateCapability"], "writable");
    assert_eq!(
        root["placement"]["stateRoot"],
        nested["placement"]["stateRoot"]
    );
    assert_ne!(
        root["placement"]["stateRoot"],
        root["placement"]["sourceRoot"]
    );
    assert_eq!(
        fs::read_to_string(f.source.join("keep.txt")).unwrap(),
        "source sentinel"
    );
    // Context must stay readable when both source and private binding metadata
    // are read-only: reader must not chmod the binding directory.
    fs::set_permissions(&f.bindings, fs::Permissions::from_mode(0o500)).unwrap();
    assert_eq!(f.context(&f.source)["project"], root["project"]);
    fs::set_permissions(&f.bindings, fs::Permissions::from_mode(0o700)).unwrap();
}

#[test]
fn readonly_preferred_state_automatically_uses_external_owned_storage() {
    let f = Fixture::new();
    let preferred = f.source.join(".exitbind");
    fs::create_dir(&preferred).unwrap();
    fs::set_permissions(&preferred, fs::Permissions::from_mode(0o500)).unwrap();
    let out = f.call(&f.source, &["init", "--skip-skills"]);
    fs::set_permissions(&preferred, fs::Permissions::from_mode(0o700)).unwrap();
    assert!(out.status.success(), "{out:?}");
    let facts = f.context(&f.source.join("nested"));
    assert_eq!(facts["placement"]["sourceCapability"], "writable");
    assert_eq!(facts["placement"]["stateCapability"], "writable");
    assert!(!f.source.join("exitbind.json").exists());
    assert_ne!(facts["placement"]["stateRoot"], preferred.to_str().unwrap());
}

#[test]
fn gitfile_worktree_and_nested_cwd_discover_git_identity_without_mutating_metadata() {
    let f = Fixture::new();
    f.git(&["init", "-q"]);
    f.git(&[
        "-c",
        "user.name=Fixture",
        "-c",
        "user.email=fixture@example.invalid",
        "commit",
        "--allow-empty",
        "-qm",
        "fixture",
    ]);
    let tree = f.base.join("linked");
    f.git(&[
        "worktree",
        "add",
        "-q",
        "-b",
        "fixture-linked",
        tree.to_str().unwrap(),
    ]);
    fs::create_dir(tree.join("nested")).unwrap();
    let marker = fs::read(tree.join(".git")).unwrap();
    let out = f.call(
        &tree.join("nested"),
        &["init", "--mode", "auto", "--skip-skills"],
    );
    assert!(out.status.success(), "{out:?}");
    let facts = f.context(&tree.join("nested"));
    assert_eq!(facts["placement"]["sourceRoot"], tree.to_str().unwrap());
    assert_eq!(facts["placement"]["git"]["topology"], "worktree");
    assert_ne!(
        facts["placement"]["git"]["gitDir"],
        facts["placement"]["git"]["commonDir"]
    );
    assert_eq!(fs::read(tree.join(".git")).unwrap(), marker);
    assert!(!tree.join("exitbind.json").exists());
}

#[test]
fn explicit_roots_win_and_unavailable_storage_is_a_classified_refusal() {
    let f = Fixture::new();
    let control = f.base.join("chosen-control");
    let state = f.base.join("chosen-state");
    for root in [&control, &state] {
        fs::create_dir(root).unwrap();
        fs::set_permissions(root, fs::Permissions::from_mode(0o700)).unwrap();
    }
    let out = f.call(
        &f.source,
        &[
            "init",
            "--mode",
            "auto",
            "--skip-skills",
            "--project-id",
            "chosen",
            "--control-root",
            control.to_str().unwrap(),
            "--state-root",
            state.to_str().unwrap(),
        ],
    );
    assert!(out.status.success(), "{out:?}");
    let facts = f.context(&f.source.join("nested"));
    assert_eq!(facts["placement"]["stateRoot"], state.to_str().unwrap());
    assert_eq!(facts["project"]["identity"]["id"], "chosen");
    assert_eq!(facts["placement"]["git"]["state"], "unavailable");
    let other = Fixture::new();
    fs::set_permissions(&other.state, fs::Permissions::from_mode(0o500)).unwrap();
    fs::set_permissions(&other.base, fs::Permissions::from_mode(0o500)).unwrap();
    let out = other.call(&other.source, &["init", "--mode", "auto", "--skip-skills"]);
    assert!(!out.status.success());
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("unsupported storage"),
        "{out:?}"
    );
    assert!(!other.source.join("exitbind.json").exists());
    fs::set_permissions(&other.base, fs::Permissions::from_mode(0o700)).unwrap();
    fs::set_permissions(&other.state, fs::Permissions::from_mode(0o700)).unwrap();
}
