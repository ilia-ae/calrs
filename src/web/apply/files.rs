//! ilia fork: `config apply`, files moved into place after the commit (the
//! avatar).
//!
//! A section writes a new file under a temporary name in the data directory
//! and pushes a `Staged` onto `Ctx::staged`, together with the file it
//! replaces or removes. Nothing under `avatars/` changes before the
//! transaction commits: `finish` then renames each new file into place and
//! removes the old ones. On an error or a `--dry-run` it only removes the
//! temporary files, so the served files and the database stay in step.

use super::ApplyError;
use std::path::{Path, PathBuf};

/// A file change waiting for the commit.
#[derive(Debug)]
pub(crate) struct Staged(Action);

#[derive(Debug)]
enum Action {
    /// `temp`, written by this run, becomes `dest`.
    Place {
        temp: PathBuf,
        dest: PathBuf,
        sha256: String,
    },
    /// `path` is removed.
    Remove { path: PathBuf },
}

impl Staged {
    /// `temp` (whose content has the SHA-256 `sha256`) is to become `dest`.
    /// Push it before writing `temp`, so a failed write is cleaned up too.
    pub(super) fn place(temp: PathBuf, dest: PathBuf, sha256: String) -> Self {
        Self(Action::Place { temp, dest, sha256 })
    }

    /// `path` is to be removed.
    pub(super) fn remove(path: PathBuf) -> Self {
        Self(Action::Remove { path })
    }

    /// The SHA-256 of what `staged` puts at `dest` after the commit, if it
    /// puts anything there (the last placement wins).
    pub(super) fn placed_at<'a>(staged: &'a [Staged], dest: &Path) -> Option<&'a str> {
        staged.iter().rev().find_map(|s| match &s.0 {
            Action::Place {
                dest: d, sha256, ..
            } if d == dest => Some(sha256.as_str()),
            _ => None,
        })
    }
}

/// Move the staged files into place when `committed`, else remove the
/// temporary ones. An old file that cannot be removed is only logged: the
/// database no longer names it. A new file that cannot be moved into place
/// is an error, since the committed database names it.
pub(super) fn finish(staged: Vec<Staged>, committed: bool) -> Result<(), ApplyError> {
    if !committed {
        for s in &staged {
            if let Action::Place { temp, .. } = &s.0 {
                discard(temp);
            }
        }
        return Ok(());
    }
    let mut failed = None;
    let mut placed: Vec<&Path> = Vec::new();
    for s in &staged {
        if let Action::Place { temp, dest, .. } = &s.0 {
            match place(temp, dest) {
                Ok(()) => placed.push(dest),
                Err(error) => {
                    discard(temp);
                    failed.get_or_insert_with(|| {
                        ApplyError::after_commit(
                            "a file could not be moved into place",
                            format!("{}: {error}", dest.display()),
                        )
                    });
                }
            }
        }
    }
    for s in &staged {
        let Action::Remove { path } = &s.0 else {
            continue;
        };
        if placed.contains(&path.as_path()) {
            continue;
        }
        match std::fs::remove_file(path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                tracing::warn!(path = %path.display(), %error, "config apply: could not remove a replaced file");
            }
        }
    }
    failed.map_or(Ok(()), Err)
}

/// `temp` becomes `dest`, its directory created if needed. A rename where
/// it can be one, else a copy.
fn place(temp: &Path, dest: &Path) -> std::io::Result<()> {
    if let Some(dir) = dest.parent() {
        std::fs::create_dir_all(dir)?;
    }
    if std::fs::rename(temp, dest).is_ok() {
        return Ok(());
    }
    std::fs::copy(temp, dest)?;
    discard(temp);
    Ok(())
}

/// Remove a temporary file; one already gone is fine.
fn discard(temp: &Path) {
    match std::fs::remove_file(temp) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            tracing::warn!(path = %temp.display(), %error, "config apply: could not remove a temporary file");
        }
    }
}
