use std::sync::OnceLock;

use anyhow::{Context, Result, bail};
use cpan_packagedetails::{Entry, PackageDetails};
use url::Url;

use crate::distribution::Distribution;
use crate::fetch::fetch_bytes;

/// An interface to a CPAN mirror rooted at [`root_url`](Mirror::root_url).
///
/// The mirror is assumed to use the standard CPAN layout: the package index
/// at `modules/02packages.details.txt.gz` and distribution tarballs under
/// `authors/id/`.
#[derive(Debug)]
pub struct Mirror {
    root_url: Url,
    details: OnceLock<PackageDetails>,
}

impl Mirror {
    /// Create a mirror rooted at `root_url`.
    ///
    /// The URL scheme must be one of `ftp`, `http`, `https`, or `file`;
    /// anything else is an error. A trailing slash is appended to the path
    /// if missing, so that the mirror layout resolves correctly regardless
    /// of how the caller wrote the URL.
    ///
    /// No network or filesystem access happens here: the package index is
    /// fetched lazily, on the first call to [`search`](Mirror::search) or
    /// [`search_like`](Mirror::search_like).
    pub fn new(root_url: &str) -> Result<Self> {
        let mut url =
            Url::parse(root_url).with_context(|| format!("invalid root_url: {root_url}"))?;

        match url.scheme() {
            "ftp" | "http" | "https" | "file" => {}
            other => {
                bail!("unsupported root_url scheme {other:?} (expected ftp, http, https, or file)")
            }
        }

        if !url.path().ends_with('/') {
            let with_slash = format!("{}/", url.path());
            url.set_path(&with_slash);
        }

        Ok(Mirror {
            root_url: url,
            details: OnceLock::new(),
        })
    }

    /// The root URL of the mirror. Always carries a trailing slash.
    pub fn root_url(&self) -> &Url {
        &self.root_url
    }

    /// The URL of the package index, `modules/02packages.details.txt.gz`.
    pub fn index_url(&self) -> Url {
        self.root_url
            .join("modules/02packages.details.txt.gz")
            .expect("the index path is always a valid relative URL")
    }

    /// Find the distribution that provides the given Perl module.
    ///
    /// The lookup is by module name, so it resolves modules to whichever
    /// distribution ships them: searching for `FFI::Build` returns the
    /// `FFI-Platypus` distribution, because that is where `FFI::Build`
    /// lives.
    ///
    /// Returns `Ok(None)` when no module by that name is in the index. The
    /// tarball is **not** downloaded; use [`Distribution::save_as`] for that.
    pub fn search(&self, module: &str) -> Result<Option<Distribution>> {
        let details = self.details()?;
        details
            .get(module)
            .map(|entry| self.distribution_for(entry))
            .transpose()
    }

    /// Find every distribution whose providing module name matches a SQL
    /// `LIKE` pattern (`%` matches any run of characters, `_` any single
    /// character), e.g. `"FFI::Platypus::%"`.
    ///
    /// As with [`search`](Mirror::search), no tarballs are downloaded.
    pub fn search_like(&self, pattern: &str) -> Result<Vec<Distribution>> {
        let details = self.details()?;
        details
            .search_like(pattern)?
            .into_iter()
            .map(|entry| self.distribution_for(entry))
            .collect()
    }

    fn distribution_for(&self, entry: &Entry) -> Result<Distribution> {
        let path = entry.path().to_string();
        let filename = path
            .rsplit('/')
            .find(|segment| !segment.is_empty())
            .unwrap_or(&path)
            .to_string();
        let url = self
            .root_url
            .join(&format!("authors/id/{path}"))
            .with_context(|| format!("could not build a distribution URL for {path}"))?;
        Ok(Distribution::new(filename, path, url))
    }

    /// Lazily fetch and parse the package index, caching it for reuse.
    fn details(&self) -> Result<&PackageDetails> {
        if let Some(details) = self.details.get() {
            return Ok(details);
        }

        let index_url = self.index_url();
        let bytes = fetch_bytes(&index_url)
            .with_context(|| format!("failed to fetch the package index: {index_url}"))?;
        let parsed =
            PackageDetails::load_bytes(&bytes).context("failed to parse 02packages.details.txt")?;

        let _ = self.details.set(parsed);
        Ok(self.details.get().expect("package index was just set"))
    }
}
