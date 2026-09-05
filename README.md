# cpan-mirror

An interface to a [CPAN](https://www.cpan.org/) mirror.

A `Mirror` is rooted at a `root_url` and understands the standard CPAN
layout: the package index lives at `modules/02packages.details.txt.gz` and
distribution tarballs live under `authors/id/`. The `ftp`, `http`, `https`,
and `file` URL schemes are all supported.

Given a Perl module name, `Mirror::search` locates the `Distribution` that
provides it. The distribution knows its filename and its absolute URL on the
mirror, but the tarball itself is not downloaded until `Distribution::save_as`
is called.

## Usage

```toml
[dependencies]
cpan-mirror = { git = "https://github.com/uperl/rust-cpan-mirror" }
```

```rust
use cpan_mirror::Mirror;

fn main() -> anyhow::Result<()> {
    let mirror = Mirror::new("https://cpan.metacpan.org/")?;

    // `FFI::Build` ships inside the `FFI-Platypus` distribution, so a search
    // by module name resolves to that distribution's tarball.
    let dist = mirror.search("FFI::Build")?.expect("module not found");
    assert_eq!(dist.filename(), "FFI-Platypus-2.08.tar.gz");
    println!("{}", dist.url());

    // Nothing has been downloaded yet. This is the call that fetches; it
    // returns the path of the written file.
    let saved = dist.save_as("./downloads")?;
    println!("saved to {}", saved.display());

    Ok(())
}
```

## API

### `Mirror`

| Item | Description |
| --- | --- |
| `Mirror::new(root_url: &str) -> Result<Mirror>` | Create a mirror. The scheme must be `ftp`, `http`, `https`, or `file`. No I/O happens here. |
| `mirror.root_url() -> &Url` | The root URL, always with a trailing slash. |
| `mirror.index_url() -> Url` | The package index URL, `{root}/modules/02packages.details.txt.gz`. |
| `mirror.search(module: &str) -> Result<Option<Distribution>>` | Resolve a Perl module name to the distribution that ships it. `Ok(None)` if unknown. |
| `mirror.search_like(pattern: &str) -> Result<Vec<Distribution>>` | Resolve every module whose name matches a SQL `LIKE` pattern (`%`, `_`). |

The package index is fetched and parsed lazily on the first search, then
cached for the lifetime of the `Mirror`.

### `Distribution`

| Item | Description |
| --- | --- |
| `dist.filename() -> &str` | The tarball filename, e.g. `FFI-Platypus-2.08.tar.gz`. |
| `dist.path() -> &str` | The path relative to `authors/id/`, e.g. `P/PL/PLICEASE/FFI-Platypus-2.08.tar.gz`. |
| `dist.url() -> &Url` | The absolute URL of the tarball on the mirror. |
| `dist.save_as(dir) -> Result<PathBuf>` | Download the tarball into `dir` (created if needed) and return the written path. The only method that performs a fetch. |

## Errors

All fallible operations return `anyhow::Result`, with context attached at each
boundary (URL parsing, unsupported scheme, connect/login/retrieve failures,
index parse failures, and filesystem writes).

## License

MIT — see [LICENSE](LICENSE).
