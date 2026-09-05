//! Scheme-aware "download the whole thing into memory" helper, shared by
//! the package-index fetch and the tarball fetch.

use std::io::Read;

use anyhow::{Context, Result, anyhow, bail};
use url::Url;

/// Download the full contents of `url`, dispatching on its scheme.
///
/// Supported schemes: `http`, `https`, `ftp`, and `file`.
pub(crate) fn fetch_bytes(url: &Url) -> Result<Vec<u8>> {
    match url.scheme() {
        "http" | "https" => fetch_http(url),
        "ftp" => fetch_ftp(url),
        "file" => fetch_file(url),
        other => bail!("unsupported URL scheme {other:?} (expected http, https, ftp, or file)"),
    }
}

fn fetch_http(url: &Url) -> Result<Vec<u8>> {
    let response = ureq::get(url.as_str())
        .call()
        .with_context(|| format!("HTTP request failed: {url}"))?;
    let mut buf = Vec::new();
    response
        .into_reader()
        .read_to_end(&mut buf)
        .with_context(|| format!("failed to read HTTP response body: {url}"))?;
    Ok(buf)
}

fn fetch_ftp(url: &Url) -> Result<Vec<u8>> {
    use suppaftp::FtpStream;
    use suppaftp::types::FileType;

    let host = url
        .host_str()
        .with_context(|| format!("ftp URL has no host: {url}"))?;
    let port = url.port().unwrap_or(21);

    let mut ftp = FtpStream::connect((host, port))
        .with_context(|| format!("failed to connect to ftp://{host}:{port}"))?;
    ftp.login("anonymous", "anonymous@example.com")
        .context("anonymous FTP login failed")?;
    ftp.transfer_type(FileType::Binary)
        .context("failed to switch FTP connection to binary mode")?;

    let path = url.path();
    let data = ftp
        .retr_as_buffer(path)
        .with_context(|| format!("failed to retrieve {path} over FTP"))?;
    let _ = ftp.quit();

    Ok(data.into_inner())
}

fn fetch_file(url: &Url) -> Result<Vec<u8>> {
    let path = url
        .to_file_path()
        .map_err(|_| anyhow!("not a usable file:// path: {url}"))?;
    std::fs::read(&path).with_context(|| format!("failed to read {}", path.display()))
}
