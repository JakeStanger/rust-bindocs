use std::path::{Path, PathBuf};

pub trait PathExt<P: AsRef<Path>> {
    fn join_if_exists(self, path: P) -> PathBuf;
}

impl<P: AsRef<Path>, P2: AsRef<Path>> PathExt<P2> for P
where
    PathBuf: From<P>,
{
    fn join_if_exists(self, path: P2) -> PathBuf {
        let extended = self.as_ref().join(path);
        if extended.exists() {
            extended
        } else {
            self.into()
        }
    }
}
