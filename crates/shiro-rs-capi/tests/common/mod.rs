use std::path::{Path, PathBuf};
pub struct Directory(PathBuf);
impl Directory {
    pub fn new(label: &str) -> Self {
        let root = std::env::temp_dir().canonicalize().unwrap();
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = root.join(format!("shiro-rs-{label}-{}-{nonce}", std::process::id()));
        assert_eq!(path.parent(), Some(root.as_path()));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
    pub fn path(&self) -> &Path {
        &self.0
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
