//! An interface to a [CPAN] mirror.
//!
//! A [`Mirror`] is rooted at a `root_url` and understands the standard CPAN
//! layout: the package index lives at `modules/02packages.details.txt.gz`
//! and distribution tarballs live under `authors/id/`. The `ftp`, `http`,
//! `https`, and `file` URL schemes are all supported.
//!
//! Given a Perl module name, [`Mirror::search`] locates the [`Distribution`]
//! that provides it. The distribution knows its filename and its absolute
//! URL on the mirror, but the tarball itself is not downloaded until
//! [`Distribution::save_as`] is called.
//!
//! ```no_run
//! use cpan_mirror::Mirror;
//!
//! # fn main() -> anyhow::Result<()> {
//! let mirror = Mirror::new("https://cpan.metacpan.org/")?;
//!
//! // `FFI::Build` ships inside the `FFI-Platypus` distribution.
//! let dist = mirror.search("FFI::Build")?.expect("module not found");
//! assert_eq!(dist.filename(), "FFI-Platypus-2.08.tar.gz");
//!
//! // Nothing has been downloaded yet; this is the call that fetches.
//! let saved = dist.save_as("./downloads")?;
//! println!("saved to {}", saved.display());
//! # Ok(())
//! # }
//! ```
//!
//! [CPAN]: https://www.cpan.org/

mod distribution;
mod fetch;
mod mirror;

pub use distribution::Distribution;
pub use mirror::Mirror;
