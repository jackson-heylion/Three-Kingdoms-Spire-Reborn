//! Advisory single-writer session guard for terminal save directories.
//! The lock file must NEVER be deleted on drop: unlinking a locked inode allows
//! a second process to create a different inode and acquire an unrelated lock.
use crate::{Action, Game};
use std::{
    fs::{self, File, OpenOptions, TryLockError},
    io,
    path::Path,
};

/// Hold this value for the entire save-directory session, including load and recovery.
/// This is a cooperative local-filesystem lock, not a transaction log or fsync barrier.
pub struct SaveDirectoryLock {
    file: File,
}

impl SaveDirectoryLock {
    pub fn acquire(directory: &Path) -> io::Result<Self> {
        fs::create_dir_all(directory)?;
        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .truncate(false)
            .open(directory.join(".tks.lock"))?;
        match file.try_lock() {
            Ok(()) => Ok(Self { file }),
            Err(TryLockError::WouldBlock) => Err(io::Error::new(
                io::ErrorKind::WouldBlock,
                "当前存档目录已被另一个 TKS 实例占用；请关闭已有游戏再试",
            )),
            Err(TryLockError::Error(error)) => Err(error),
        }
    }
}

impl Drop for SaveDirectoryLock {
    fn drop(&mut self) {
        let _ = self.file.unlock();
    }
}


// Crash-consistent replacement for small JSON save documents.
// Callers must hold SaveDirectoryLock through the whole session.
// Temp data and backup rotations use same-directory synced file renames.
use std::{
    io::Write,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

struct StagedFile {
    path: PathBuf,
}

impl StagedFile {
    fn new(target: &Path, contents: &[u8]) -> io::Result<Self> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        for _ in 0..128 {
            let sequence = NEXT.fetch_add(1, Ordering::Relaxed);
            let temporary = target.with_extension(format!(
                "tmp.{}.{}", std::process::id(), sequence
            ));
            match OpenOptions::new().write(true).create_new(true).open(&temporary) {
                Ok(mut file) => {
                    let staged = Self { path: temporary };
                    file.write_all(contents)?;
                    file.sync_all()?;
                    return Ok(staged);
                }
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error),
            }
        }
        Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "too many abandoned temporary save files",
        ))
    }

    fn commit(self, target: &Path) -> io::Result<()> {
        fs::rename(&self.path, target)
    }
}

impl Drop for StagedFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn parent_directory(path: &Path) -> &Path {
    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
}

fn sync_directory(directory: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        File::open(directory)?.sync_all()
    }
    #[cfg(not(unix))]
    {
        let _ = directory;
        Ok(())
    }
}

/// Atomically replace a JSON file and rotate complete previous bytes into .bak.
/// This does not make multiple different JSON files a shared transaction.
pub fn atomic_write(path: &Path, data: &str) -> io::Result<()> {
    let directory = parent_directory(path);
    fs::create_dir_all(directory)?;
    let staged = StagedFile::new(path, data.as_bytes())?;
    if path.exists() {
        let previous = fs::read(path)?;
        let backup = path.with_extension("bak");
        StagedFile::new(&backup, &previous)?.commit(&backup)?;
        sync_directory(directory)?;
    }
    staged.commit(path)?;
    sync_directory(directory)
}

/// Apply an action only after its next snapshot has reached the save path.
/// A failed write must not advance the live game, revision, RNG or action history.
/// Profile persistence is separate and is not part of this single-file guarantee.
pub fn apply_durable_action(game: &mut Game, action: Action, path: &Path) -> Result<Vec<String>, String> {
    let mut next = game.clone();
    let messages = next.apply(action)?;
    atomic_write(path, &next.to_json()?)
        .map_err(|error| format!("战役存档失败，动作未提交：{error}"))?;
    *game = next;
    Ok(messages)
}

/// Restore already-validated backup bytes without rotating corrupted bytes to .bak.
/// Caller quarantines the invalid primary as .corrupt before this operation.
pub fn restore_primary_from_backup(path: &Path, verified_data: &str) -> io::Result<()> {
    let directory = parent_directory(path);
    fs::create_dir_all(directory)?;
    StagedFile::new(path, verified_data.as_bytes())?.commit(path)?;
    sync_directory(directory)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    fn directory() -> PathBuf {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        std::env::temp_dir().join(format!(
            "tks-single-writer-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ))
    }

    #[test]
    fn concurrent_writers_fail_closed_without_removing_the_file() {
        let dir = directory();
        let first = SaveDirectoryLock::acquire(&dir).unwrap();
        let error = SaveDirectoryLock::acquire(&dir).err().expect("second writer must fail");
        assert_eq!(error.kind(), io::ErrorKind::WouldBlock);
        assert!(dir.join(".tks.lock").exists());
        drop(first);
        let next = SaveDirectoryLock::acquire(&dir).unwrap();
        drop(next);
        assert!(dir.join(".tks.lock").exists());
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn different_directories_are_independent() {
        let first = directory();
        let second = directory();
        let guard_a = SaveDirectoryLock::acquire(&first).unwrap();
        let guard_b = SaveDirectoryLock::acquire(&second).unwrap();
        drop(guard_b);
        drop(guard_a);
        fs::remove_dir_all(first).unwrap();
        fs::remove_dir_all(second).unwrap();
    }

    #[test]
    fn a_file_instead_of_a_directory_is_an_error() {
        let dir = directory();
        fs::create_dir_all(&dir).unwrap();
        let file_path = dir.join("not-a-directory");
        fs::write(&file_path, "x").unwrap();
        assert!(SaveDirectoryLock::acquire(&file_path).is_err());
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn atomic_write_rotates_only_complete_prior_versions() {
        let dir = directory();
        let primary = dir.join("save.json");
        atomic_write(&primary, r#"{"revision":1}"#).unwrap();
        assert_eq!(fs::read_to_string(&primary).unwrap(), r#"{"revision":1}"#);
        assert!(!dir.join("save.bak").exists());
        atomic_write(&primary, r#"{"revision":2}"#).unwrap();
        assert_eq!(fs::read_to_string(&primary).unwrap(), r#"{"revision":2}"#);
        assert_eq!(fs::read_to_string(dir.join("save.bak")).unwrap(), r#"{"revision":1}"#);
        atomic_write(&primary, r#"{"revision":3}"#).unwrap();
        assert_eq!(fs::read_to_string(&primary).unwrap(), r#"{"revision":3}"#);
        assert_eq!(fs::read_to_string(dir.join("save.bak")).unwrap(), r#"{"revision":2}"#);
        assert!(fs::read_dir(&dir).unwrap().all(|entry| {
            !entry.unwrap().file_name().to_string_lossy().contains(".tmp.")
        }));
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn failed_backup_rename_leaves_primary_intact_and_removes_staged_files() {
        let dir = directory();
        let primary = dir.join("save.json");
        atomic_write(&primary, "old").unwrap();
        fs::create_dir(dir.join("save.bak")).unwrap();
        assert!(atomic_write(&primary, "new").is_err());
        assert_eq!(fs::read_to_string(&primary).unwrap(), "old");
        assert!(fs::read_dir(&dir).unwrap().all(|entry| {
            !entry.unwrap().file_name().to_string_lossy().contains(".tmp.")
        }));
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn failed_durable_action_preserves_live_game_rng_history_and_disk() {
        let dir = directory();
        let primary = dir.join("save.json");
        let mut game = Game::new(crate::content::Hero::GuanYu, 96100);
        let before = game.to_json().unwrap();
        atomic_write(&primary, &before).unwrap();
        fs::create_dir(dir.join("save.bak")).unwrap();
        let error = apply_durable_action(&mut game, Action::Choose { index: 1 }, &primary)
            .unwrap_err();
        assert!(error.contains("战役存档失败"), "{error}");
        assert_eq!(game.to_json().unwrap(), before);
        assert_eq!(fs::read_to_string(&primary).unwrap(), before);
        fs::remove_dir(dir.join("save.bak")).unwrap();
        apply_durable_action(&mut game, Action::Choose { index: 1 }, &primary).unwrap();
        assert_eq!(game.revision, 1);
        assert_eq!(game.stage, crate::Stage::Combat);
        assert_eq!(fs::read_to_string(&primary).unwrap(), game.to_json().unwrap());
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn rejected_durable_action_does_not_touch_save_or_backup() {
        let dir = directory();
        let primary = dir.join("save.json");
        let mut game = Game::new(crate::content::Hero::GuanYu, 96101);
        let before = game.to_json().unwrap();
        atomic_write(&primary, &before).unwrap();
        assert!(apply_durable_action(&mut game, Action::Choose { index: 99 }, &primary).is_err());
        assert_eq!(game.to_json().unwrap(), before);
        assert_eq!(fs::read_to_string(&primary).unwrap(), before);
        assert!(!dir.join("save.bak").exists());
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn recovery_never_clobbers_the_known_good_backup() {
        let dir = directory();
        let primary = dir.join("save.json");
        atomic_write(&primary, "valid-1").unwrap();
        atomic_write(&primary, "valid-2").unwrap();
        fs::write(&primary, "{damaged").unwrap();
        fs::rename(&primary, primary.with_extension("corrupt")).unwrap();
        restore_primary_from_backup(&primary, "valid-1").unwrap();
        assert_eq!(fs::read_to_string(&primary).unwrap(), "valid-1");
        assert_eq!(fs::read_to_string(dir.join("save.bak")).unwrap(), "valid-1");
        assert_eq!(fs::read_to_string(dir.join("save.corrupt")).unwrap(), "{damaged");
        fs::remove_dir_all(dir).unwrap();
    }

}
