//! Typed classification of remote-server install failures.
//!
//! Production install errors arrive as raw exit codes + stderr strings
//! from the install script or SSH transport layer.  This module converts
//! them into a [`InstallFailureCategory`] enum so that:
//!
//! 1. Telemetry gets a stable, enumerated tag instead of a free-form string.
//! 2. The retry/fallback logic can match on categories instead of fragile
//!    substring tests.
//! 3. UI can render targeted user-facing messages.
//!
//! The classifier is intentionally conservative: if the stderr doesn't
//! match any known pattern, it falls through to [`InstallFailureCategory::Unknown`].

use std::fmt;

/// Exit code the install script uses when the detected architecture is
/// unsupported (e.g. `mips`, `ppc64le`).
const UNSUPPORTED_ARCH_OR_OS_EXIT_CODE: i32 = 2;

/// Typed classification of a remote-server install failure.
///
/// Each variant corresponds to one of the CSV failure families observed
/// in production.  The ordering roughly follows the install script's
/// execution flow: platform checks → download → extraction → placement.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InstallFailureCategory {
    // ── Platform / environment ──────────────────────────────────────
    /// `uname -m` reported an architecture we don't ship a binary for.
    UnsupportedArch { arch: String },
    /// `uname -s` reported an OS we don't support (e.g. FreeBSD).
    UnsupportedOs { os: String },
    /// Neither `curl` nor `wget` is available.  This is the trigger for
    /// the SCP upload fallback in the SSH transport.
    NoHttpClient,
    /// `tar` is not available on the remote host.
    MissingTar,
    /// `bash` is not available (script is piped into `bash -s`).
    MissingBash,

    // ── Network / download ──────────────────────────────────────────
    /// DNS resolution failed.
    DnsFailure,
    /// TCP connection was refused or the host was unreachable.
    ConnectionRefused,
    /// TLS handshake failed (certificate validation, expired cert, etc.).
    TlsCaFailure,
    /// HTTP 403 Forbidden from the download endpoint.
    HttpForbidden,
    /// HTTP 502 Bad Gateway (or similar server error) from the CDN.
    HttpServerError { status_code: u16 },
    /// The download started but was truncated (curl exit 18 / wget
    /// partial content).
    PartialDownload,
    /// The install script timed out (SSH-level timeout fired before
    /// the script exited).
    InstallTimeout,

    // ── Filesystem / extraction ─────────────────────────────────────
    /// Writing the downloaded tarball to disk failed (e.g. broken pipe
    /// to the temp file, I/O error).
    DownloadWriteFailure,
    /// `mkdir -p` or `mv` on the install directory failed with EACCES.
    InstallDirPermissionDenied,
    /// No space left on device (ENOSPC) or disk quota exceeded.
    NoSpaceOrQuota,
    /// The filesystem (or mount) is read-only.
    ReadOnlyFilesystem,
    /// `tar -xzf` failed due to ownership or permission errors.
    TarPermissionFailure,

    // ── SSH / auth ──────────────────────────────────────────────────
    /// The remote requires a password change or has no TTY for
    /// interactive auth prompts.
    ExpiredPasswordOrNoTty,
    /// Permission denied writing to a startup file (e.g. ~/.bashrc is
    /// read-only or owned by root).
    StartupFilePermissionDenied,
    /// SSH exited with code 255, indicating a forced disconnect, broken
    /// pipe, or connection reset.
    SshDisconnect,

    // ── Catch-all ───────────────────────────────────────────────────
    /// The error didn't match any known pattern.
    Unknown,
}

impl InstallFailureCategory {
    /// Whether this failure category is potentially retriable.
    ///
    /// Categories caused by transient conditions (network hiccups,
    /// timeouts, server errors) return `true`.  Permanent host
    /// conditions (permissions, disk, auth, architecture) return
    /// `false` to prevent wasteful blind retries.
    pub fn is_retriable(&self) -> bool {
        match self {
            // Transient / network
            Self::DnsFailure
            | Self::ConnectionRefused
            | Self::TlsCaFailure
            | Self::HttpServerError { .. }
            | Self::PartialDownload
            | Self::InstallTimeout => true,

            // Permanent host condition — do NOT retry
            Self::UnsupportedArch { .. }
            | Self::UnsupportedOs { .. }
            | Self::NoHttpClient
            | Self::MissingTar
            | Self::MissingBash
            | Self::HttpForbidden
            | Self::DownloadWriteFailure
            | Self::InstallDirPermissionDenied
            | Self::NoSpaceOrQuota
            | Self::ReadOnlyFilesystem
            | Self::TarPermissionFailure
            | Self::ExpiredPasswordOrNoTty
            | Self::StartupFilePermissionDenied
            | Self::SshDisconnect
            | Self::Unknown => false,
        }
    }

    /// Returns a short, stable string tag suitable for telemetry.
    pub fn telemetry_tag(&self) -> &'static str {
        match self {
            Self::UnsupportedArch { .. } => "unsupported_arch",
            Self::UnsupportedOs { .. } => "unsupported_os",
            Self::NoHttpClient => "no_http_client",
            Self::MissingTar => "missing_tar",
            Self::MissingBash => "missing_bash",
            Self::DnsFailure => "dns_failure",
            Self::ConnectionRefused => "connection_refused",
            Self::TlsCaFailure => "tls_ca_failure",
            Self::HttpForbidden => "http_forbidden",
            Self::HttpServerError { .. } => "http_server_error",
            Self::PartialDownload => "partial_download",
            Self::InstallTimeout => "install_timeout",
            Self::DownloadWriteFailure => "download_write_failure",
            Self::InstallDirPermissionDenied => "install_dir_permission_denied",
            Self::NoSpaceOrQuota => "no_space_or_quota",
            Self::ReadOnlyFilesystem => "read_only_filesystem",
            Self::TarPermissionFailure => "tar_permission_failure",
            Self::ExpiredPasswordOrNoTty => "expired_password_or_no_tty",
            Self::StartupFilePermissionDenied => "startup_file_permission_denied",
            Self::SshDisconnect => "ssh_disconnect",
            Self::Unknown => "unknown",
        }
    }
}

impl fmt::Display for InstallFailureCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedArch { arch } => write!(f, "unsupported architecture: {arch}"),
            Self::UnsupportedOs { os } => write!(f, "unsupported OS: {os}"),
            Self::NoHttpClient => write!(f, "neither curl nor wget is available"),
            Self::MissingTar => write!(f, "tar is not installed on the remote host"),
            Self::MissingBash => write!(f, "bash is not available on the remote host"),
            Self::DnsFailure => write!(f, "DNS resolution failed"),
            Self::ConnectionRefused => write!(f, "connection refused or host unreachable"),
            Self::TlsCaFailure => write!(f, "TLS/certificate verification failed"),
            Self::HttpForbidden => write!(f, "HTTP 403 Forbidden"),
            Self::HttpServerError { status_code } => write!(f, "HTTP server error ({status_code})"),
            Self::PartialDownload => write!(f, "download was truncated or incomplete"),
            Self::InstallTimeout => write!(f, "install timed out"),
            Self::DownloadWriteFailure => write!(f, "failed to write downloaded file"),
            Self::InstallDirPermissionDenied => {
                write!(f, "permission denied on install directory")
            }
            Self::NoSpaceOrQuota => write!(f, "no space left on device or quota exceeded"),
            Self::ReadOnlyFilesystem => write!(f, "read-only filesystem"),
            Self::TarPermissionFailure => write!(f, "tar extraction failed (permission error)"),
            Self::ExpiredPasswordOrNoTty => {
                write!(f, "expired password or no TTY for interactive auth")
            }
            Self::StartupFilePermissionDenied => {
                write!(f, "permission denied writing startup file")
            }
            Self::SshDisconnect => write!(f, "SSH connection was forcibly closed (exit 255)"),
            Self::Unknown => write!(f, "unknown install failure"),
        }
    }
}

/// Classify a raw install failure from the install script or SSH transport
/// into a typed [`InstallFailureCategory`].
///
/// # Arguments
///
/// * `exit_code` — The process exit code, if available. `None` when the
///   process was killed by a signal or the exit code wasn't captured.
/// * `stderr` — Combined stderr output from the install script / SSH
///   command.
/// * `is_timeout` — `true` when the failure was caused by the
///   SSH-level timeout firing before the script exited (the async
///   runtime kills the child and returns a timeout error).
pub fn classify_install_failure(
    exit_code: Option<i32>,
    stderr: &str,
    is_timeout: bool,
) -> InstallFailureCategory {
    // Timeout is unambiguous — the script didn't finish in time.
    if is_timeout {
        return InstallFailureCategory::InstallTimeout;
    }

    // SSH exit 255 → forced disconnect / broken pipe.
    if exit_code == Some(255) {
        return InstallFailureCategory::SshDisconnect;
    }

    // Script exit 3 → no HTTP client (sentinel from install script).
    if exit_code == Some(super::NO_HTTP_CLIENT_EXIT_CODE) {
        return InstallFailureCategory::NoHttpClient;
    }

    // Script exit 2 → unsupported arch or OS.
    if exit_code == Some(UNSUPPORTED_ARCH_OR_OS_EXIT_CODE) {
        if let Some(arch) = extract_unsupported_arch(stderr) {
            return InstallFailureCategory::UnsupportedArch { arch };
        }
        if let Some(os) = extract_unsupported_os(stderr) {
            return InstallFailureCategory::UnsupportedOs { os };
        }
        // Exit 2 but no parseable arch/os — still treat as unsupported.
        return InstallFailureCategory::UnsupportedArch {
            arch: "unknown".to_string(),
        };
    }

    let lower = stderr.to_lowercase();

    // ── Bash / shell availability ───────────────────────────────────
    if lower.contains("bash: not found")
        || lower.contains("bash: command not found")
        || lower.contains("no such file or directory: bash")
        || lower.contains("cannot execute binary file") && lower.contains("bash")
    {
        return InstallFailureCategory::MissingBash;
    }

    // ── tar availability ────────────────────────────────────────────
    if lower.contains("tar: not found")
        || lower.contains("tar: command not found")
        || (lower.contains("no such file") && lower.contains("tar"))
    {
        return InstallFailureCategory::MissingTar;
    }

    // ── DNS failure ─────────────────────────────────────────────────
    if lower.contains("could not resolve host")
        || lower.contains("name or service not known")
        || lower.contains("temporary failure in name resolution")
        || lower.contains("unable to resolve host")
        || lower.contains("dns_error")
    {
        return InstallFailureCategory::DnsFailure;
    }

    // ── Connection refused / unreachable ────────────────────────────
    if lower.contains("connection refused")
        || lower.contains("no route to host")
        || lower.contains("network is unreachable")
        || lower.contains("connection timed out")
            && !lower.contains("ssl")
            && !lower.contains("tls")
    {
        return InstallFailureCategory::ConnectionRefused;
    }

    // ── TLS / CA failures ───────────────────────────────────────────
    if lower.contains("ssl")
        || lower.contains("certificate")
        || lower.contains("tls")
        || lower.contains("ca-bundle")
        || lower.contains("unable to get local issuer certificate")
        || lower.contains("verify failed")
    {
        return InstallFailureCategory::TlsCaFailure;
    }

    // ── HTTP status codes ───────────────────────────────────────────
    if lower.contains("403 forbidden")
        || lower.contains("http/1.1 403")
        || lower.contains("http/2 403")
    {
        return InstallFailureCategory::HttpForbidden;
    }
    if lower.contains("502 bad gateway")
        || lower.contains("http/1.1 502")
        || lower.contains("http/2 502")
    {
        return InstallFailureCategory::HttpServerError { status_code: 502 };
    }
    if lower.contains("503 service unavailable") || lower.contains("http/1.1 503") {
        return InstallFailureCategory::HttpServerError { status_code: 503 };
    }
    // curl exit 22 = HTTP error ≥ 400 (with -f flag)
    if exit_code == Some(22) && lower.contains("403") {
        return InstallFailureCategory::HttpForbidden;
    }
    if exit_code == Some(22) {
        // Generic HTTP server error from curl -f
        return InstallFailureCategory::HttpServerError { status_code: 0 };
    }

    // ── Partial download ────────────────────────────────────────────
    // curl exit 18 = transfer closed with outstanding read data remaining
    if exit_code == Some(18)
        || lower.contains("partial file")
        || lower.contains("transfer closed with outstanding read data")
        || lower.contains("incomplete download")
        || (lower.contains("unexpected end") && lower.contains("gz"))
    {
        return InstallFailureCategory::PartialDownload;
    }

    // ── Download write failure ──────────────────────────────────────
    // curl exit 23 = write error (e.g. broken pipe, I/O error on
    // the temp file)
    if exit_code == Some(23)
        || (lower.contains("write error") && lower.contains("download"))
        || lower.contains("failed writing body")
    {
        return InstallFailureCategory::DownloadWriteFailure;
    }

    // ── tar extraction failures ─────────────────────────────────────
    // Check tar-specific patterns *before* the generic "permission
    // denied" rule so that `tar: Cannot open: Permission denied` is
    // classified as a tar failure, not a generic install-dir EACCES.
    if (lower.contains("tar") || lower.contains("extract"))
        && (lower.contains("cannot open")
            || lower.contains("operation not permitted")
            || lower.contains("cannot change ownership")
            || lower.contains("permission denied"))
    {
        return InstallFailureCategory::TarPermissionFailure;
    }

    // ── Filesystem: permission denied ───────────────────────────────────
    if lower.contains("permission denied") {
        // Distinguish install-dir EACCES from startup-file EACCES.
        if lower.contains(".bashrc")
            || lower.contains(".bash_profile")
            || lower.contains(".profile")
            || lower.contains(".zshrc")
        {
            return InstallFailureCategory::StartupFilePermissionDenied;
        }
        return InstallFailureCategory::InstallDirPermissionDenied;
    }

    // ── Filesystem: no space / quota ────────────────────────────────
    if lower.contains("no space left on device")
        || lower.contains("disk quota exceeded")
        || lower.contains("enospc")
    {
        return InstallFailureCategory::NoSpaceOrQuota;
    }

    // ── Filesystem: read-only ───────────────────────────────────────
    if lower.contains("read-only file system") || lower.contains("erofs") {
        return InstallFailureCategory::ReadOnlyFilesystem;
    }

    // ── Expired password / no TTY ───────────────────────────────────
    if lower.contains("password has expired")
        || lower.contains("you must change your password")
        || lower.contains("no tty present")
        || lower.contains("password change required")
    {
        return InstallFailureCategory::ExpiredPasswordOrNoTty;
    }

    InstallFailureCategory::Unknown
}

/// Extracts the architecture name from an "unsupported arch: XYZ" stderr line.
fn extract_unsupported_arch(stderr: &str) -> Option<String> {
    for line in stderr.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("unsupported arch:") {
            let arch = rest.trim().to_string();
            if !arch.is_empty() {
                return Some(arch);
            }
        }
    }
    None
}

/// Extracts the OS name from an "unsupported OS: XYZ" stderr line.
fn extract_unsupported_os(stderr: &str) -> Option<String> {
    for line in stderr.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("unsupported OS:") {
            let os = rest.trim().to_string();
            if !os.is_empty() {
                return Some(os);
            }
        }
    }
    None
}

#[cfg(test)]
#[path = "install_error_tests.rs"]
mod tests;
