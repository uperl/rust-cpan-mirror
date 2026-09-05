use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use url::Url;

use crate::fetch::fetch_bytes;

/// A CPAN distribution located on a [`Mirror`](crate::Mirror).
///
/// Produced by [`Mirror::search`](crate::Mirror::search). A `Distribution`
/// only records *where* the tarball is; nothing is downloaded until
/// [`save_as`](Distribution::save_as) is called.
#[derive(Debug, Clone)]
pub struct Distribution {
    filename: String,
    path: String,
    url: Url,
}

impl Distribution {
    pub(crate) fn new(filename: String, path: String, url: Url) -> Self {
        Distribution {
            filename,
            path,
            url,
        }
    }

    /// The tarball filename, e.g. `FFI-Platypus-2.08.tar.gz`.
    pub fn filename(&self) -> &str {
        &self.filename
    }

    /// The distribution path relative to `authors/id/`, e.g.
    /// `P/PL/PLICEASE/FFI-Platypus-2.08.tar.gz`.
    pub fn path(&self) -> &str {
        &self.path
    }

    /// The absolute URL of the tarball on the mirror.
    pub fn url(&self) -> &Url {
        &self.url
    }

    /// Download the tarball and write it into `dir`, returning the path of
    /// the file written (`dir` joined with [`filename`](Self::filename)).
    ///
    /// `dir` is created (recursively) if it does not already exist. This is
    /// the only method on `Distribution` that performs a fetch.
    pub fn save_as(&self, dir: impl AsRef<Path>) -> Result<PathBuf> {
        let dir = dir.as_ref();
        std::fs::create_dir_all(dir)
            .with_context(|| format!("failed to create directory {}", dir.display()))?;

        let dest = dir.join(&self.filename);
        let bytes =
            fetch_bytes(&self.url).with_context(|| format!("failed to download {}", self.url))?;
        std::fs::write(&dest, &bytes)
            .with_context(|| format!("failed to write {}", dest.display()))?;

        Ok(dest)
    }
}
