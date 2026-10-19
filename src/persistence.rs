//! Advisory single-writer session guard for terminal save directories.
//! The lock file must NEVER be deleted on drop: unlinking a locked inode allows
//! a second process to create a different inode and acquire an unrelated lock.
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
}
