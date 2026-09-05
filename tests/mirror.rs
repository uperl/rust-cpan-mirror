use std::fs;
use std::path::PathBuf;

use cpan_mirror::Mirror;

fn fixture_root() -> String {
    format!("file://{}/tests/fixtures/", env!("CARGO_MANIFEST_DIR"))
}

fn fixture_path(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(rel)
}

fn scratch_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("cpan-mirror-test-{tag}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    dir
}

#[test]
fn rejects_unsupported_scheme() {
    let err = Mirror::new("gopher://example.com/cpan/").unwrap_err();
    assert!(
        err.to_string().contains("unsupported"),
        "unexpected error: {err}"
    );
}

#[test]
fn accepts_all_documented_schemes() {
    for url in [
        "ftp://ftp.example/CPAN/",
        "http://cpan.example/",
        "https://cpan.example/",
        "file:///srv/minicpan/",
    ] {
        assert!(Mirror::new(url).is_ok(), "{url} should be accepted");
    }
}

#[test]
fn normalises_root_url_with_trailing_slash() -> anyhow::Result<()> {
    let mirror = Mirror::new("https://cpan.example/pub/CPAN")?;
    assert_eq!(mirror.root_url().as_str(), "https://cpan.example/pub/CPAN/");
    assert_eq!(
        mirror.index_url().as_str(),
        "https://cpan.example/pub/CPAN/modules/02packages.details.txt.gz"
    );
    Ok(())
}

#[test]
fn finds_the_dist_that_ships_a_module() -> anyhow::Result<()> {
    let mirror = Mirror::new(&fixture_root())?;

    let dist = mirror
        .search("FFI::Build")?
        .expect("FFI::Build should resolve to FFI-Platypus");

    assert_eq!(dist.filename(), "FFI-Platypus-2.08.tar.gz");
    assert_eq!(dist.path(), "P/PL/PLICEASE/FFI-Platypus-2.08.tar.gz");
    assert_eq!(
        dist.url().as_str(),
        format!(
            "{}authors/id/P/PL/PLICEASE/FFI-Platypus-2.08.tar.gz",
            fixture_root()
        )
    );
    Ok(())
}

#[test]
fn unknown_module_is_none() -> anyhow::Result<()> {
    let mirror = Mirror::new(&fixture_root())?;
    assert!(mirror.search("No::Such::Thing")?.is_none());
    Ok(())
}

#[test]
fn search_does_not_fetch_the_tarball() -> anyhow::Result<()> {
    let mirror = Mirror::new(&fixture_root())?;

    // `Ghost::Module` is in the index, but its tarball is intentionally
    // absent from the fixtures. `search` still succeeds, which proves it
    // never tried to fetch.
    let dist = mirror
        .search("Ghost::Module")?
        .expect("Ghost::Module is listed in the index");
    assert_eq!(dist.filename(), "Ghost-Module-1.00.tar.gz");

    // Only now, on save_as, does the missing tarball become an error.
    let err = dist.save_as(scratch_dir("ghost")).unwrap_err();
    assert!(
        err.to_string().contains("failed to download"),
        "unexpected error: {err}"
    );
    Ok(())
}

#[test]
fn save_as_downloads_and_writes_the_tarball() -> anyhow::Result<()> {
    let mirror = Mirror::new(&fixture_root())?;
    let dist = mirror.search("FFI::Platypus")?.expect("in the index");

    let dir = scratch_dir("save");
    let written = dist.save_as(&dir)?;

    assert_eq!(written, dir.join("FFI-Platypus-2.08.tar.gz"));
    assert_eq!(
        fs::read(&written)?,
        fs::read(fixture_path(
            "authors/id/P/PL/PLICEASE/FFI-Platypus-2.08.tar.gz"
        ))?
    );

    fs::remove_dir_all(&dir)?;
    Ok(())
}
