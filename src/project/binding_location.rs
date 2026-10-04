//! Select one authoritative machine-local registry without silently forking it.
use std::{fs, path::PathBuf};

pub(super) fn select() -> Result<PathBuf, String> {
    let exitbind = super::layout_types::exitbind_surface();
    let explicit = if exitbind {
        std::env::var_os("EXITBIND_BINDINGS_DIR")
            .or_else(|| std::env::var_os("SOULMATE_BINDINGS_DIR"))
    } else {
        std::env::var_os("SOULMATE_BINDINGS_DIR")
    };
    if let Some(path) = explicit {
        return Ok(path.into());
    }
    let caller = if exitbind { "exitbind" } else { "soulmate" };
    let mut candidates = Vec::new();
    if let Some(base) = std::env::var_os("XDG_STATE_HOME") {
        candidates.push(PathBuf::from(base).join(caller).join("bindings"));
    }
    if let Some(home) = std::env::var_os("HOME") {
        let fallback = PathBuf::from(home)
            .join(".local/state")
            .join(caller)
            .join("bindings");
        if !candidates.contains(&fallback) {
            candidates.push(fallback);
        }
    }
    if candidates.iter().any(|path| !path.is_absolute()) {
        return Err("machine-local binding directory must be absolute".into());
    }
    let mut existing = Vec::new();
    for candidate in &candidates {
        match fs::symlink_metadata(candidate) {
            Ok(_) => existing.push(candidate),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(format!("binding directory: {error}")),
        }
    }
    match existing.as_slice() {
        [single] => return Ok((*single).clone()),
        [] => {}
        _ => return Err("multiple machine-local binding registries exist; select the authoritative one with the bindings-directory override".into()),
    }
    // An existing fallback stays authoritative even when the preferred root
    // becomes writable later. Only a first bootstrap selects fresh storage.
    for candidate in candidates {
        let mut ancestor = candidate.as_path();
        while !ancestor.exists() {
            ancestor = ancestor.parent().ok_or("binding directory has no parent")?;
        }
        if super::portability::writable(ancestor) {
            return Ok(candidate);
        }
    }
    Err("unsupported storage: no writable machine-local binding registry; provide an owned bindings-directory override".into())
}
