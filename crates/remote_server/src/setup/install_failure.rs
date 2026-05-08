//! Classification of remote-server install failures.
//!
//! Converts unstructured SSH stderr / exit-code pairs into a small
//! [`InstallFailureKind`] enum that downstream code can match on for
//! targeted diagnostics, telemetry, and fall-back decisions.
//!
//! Design constraints:
//! - Pattern matching is intentionally case-insensitive and substring-based
//!   so it survives locale differences and minor message rewording across
//!   coreutils / OpenSSH versions.
//! - Classification is conservative: a string that doesn't match any known
//!   pattern falls through to [`InstallFailureKind::Unclassified`] rather
//!   than being force-fit into a category.

use std::fmt;

use serde::Serialize;

/// Structured classification of a remote-server install failure.
///
/// Each variant maps to a family of errors that share a common root
/// cause and recovery path. The `Display` impl produces a short,
/// user-facing diagnostic; the `Serialize` impl is used for telemetry.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InstallFailureKind {
    /// The remote host's OS or architecture is not supported by the
    /// prebuilt binary. Treat identically to the existing
    /// [`super::UnsupportedReason`] preinstall path — fall back to
    /// legacy SSH without surfacing an error.
    UnsupportedPlatform {
        /// Short description, e.g. `"unsupported arch: mips"`.
        detail: String,
    },
    /// The install directory (or a parent) cannot be created or written
    /// because the current user lacks filesystem permissions.
    PermissionDenied,
    /// The target filesystem is mounted read-only (e.g. a locked-down
    /// container or a host in single-user mode).
    ReadOnlyFilesystem,
    /// The filesystem has no free space or the user's quota is exhausted.
    DiskFull,
    /// The remote account is in a degraded state — password expired,
    /// forced password change, or the SSH session has no controlling
    /// TTY required by the login sequence.
    AccountIssue {
        /// Short description, e.g. `"password expired"`.
        detail: String,
    },
    /// The SSH transport itself failed — connection reset, broken pipe,
    /// process killed by signal, or the SSH process exited with code 255
    /// (connection-level error).
    SshTransportError {
        /// Short description, e.g. `"connection reset by peer"`.
        detail: String,
    },
    /// The install script (or SCP upload) exceeded its deadline.
    Timeout,
    /// None of the known patterns matched. Carries the original error
    /// text so callers can still log / display it.
    Unclassified { message: String },
}

impl fmt::Display for InstallFailureKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedPlatform { detail } => write!(f, "unsupported platform: {detail}"),
            Self::PermissionDenied => write!(f, "permission denied on install directory"),
            Self::ReadOnlyFilesystem => write!(f, "read-only filesystem"),
            Self::DiskFull => write!(f, "no space left on device or disk quota exceeded"),
            Self::AccountIssue { detail } => write!(f, "account issue: {detail}"),
            Self::SshTransportError { detail } => write!(f, "SSH transport error: {detail}"),
            Self::Timeout => write!(f, "install timed out"),
            Self::Unclassified { message } => write!(f, "{message}"),
        }
    }
}

impl InstallFailureKind {
    /// Whether this failure should be treated like the existing
    /// `Unsupported` preinstall path — a clean fall-back to legacy SSH
    /// rather than a user-visible error.
    pub fn is_unsupported_platform(&self) -> bool {
        matches!(self, Self::UnsupportedPlatform { .. })
    }

    /// Whether a retry or alternative install path (e.g. SCP upload)
    /// could plausibly succeed. Transient SSH errors and timeouts are
    /// retriable; filesystem permission/space issues are not.
    pub fn is_retriable(&self) -> bool {
        matches!(
            self,
            Self::SshTransportError { .. } | Self::Timeout | Self::Unclassified { .. }
        )
    }
}

/// Classify an install failure from its stderr output and exit code.
///
/// `stderr` is the combined stderr captured from the install script
/// (or the SCP upload fallback). `exit_code` is the process exit code
/// when available (`None` for signal kills or when the code cannot be
/// determined).
///
/// The function tries each pattern family in priority order (most
/// specific first) and returns the first match. Patterns are
/// case-insensitive substring checks.
pub fn classify_install_error(stderr: &str, exit_code: Option<i32>) -> InstallFailureKind {
    let lower = stderr.to_ascii_lowercase();

    // --- SSH transport / process-level failures (check first because
    //     these can wrap other messages) ---

    // SSH exit code 255 is the canonical "connection-level error"
    // indicator from OpenSSH.
    if exit_code == Some(255) {
        let detail = extract_ssh_detail(&lower);
        return InstallFailureKind::SshTransportError { detail };
    }

    // Signal kills (exit_code is None with the process killed).
    if exit_code.is_none()
        && !stderr.is_empty()
        && contains_any(
            &lower,
            &[
                "broken pipe",
                "connection reset",
                "connection closed",
                "connection timed out",
                "connection refused",
            ],
        )
    {
        let detail = extract_ssh_detail(&lower);
        return InstallFailureKind::SshTransportError { detail };
    }

    // Explicit SSH transport keywords anywhere in stderr.
    if contains_any(
        &lower,
        &[
            "broken pipe",
            "connection reset by peer",
            "connection closed by remote host",
            "ssh_exchange_identification",
            "kex_exchange_identification",
            "packet_write_wait",
            "write failed: broken pipe",
        ],
    ) {
        let detail = extract_ssh_detail(&lower);
        return InstallFailureKind::SshTransportError { detail };
    }

    // --- Timeout ---
    if contains_any(
        &lower,
        &[
            "timed out",
            "operation timed out",
            "connection timed out",
            "timeout",
        ],
    ) && !contains_any(&lower, &["connection reset", "broken pipe"])
    {
        return InstallFailureKind::Timeout;
    }

    // --- Unsupported platform (from install script stderr) ---
    if contains_any(&lower, &["unsupported arch:", "unsupported os:"]) {
        let detail = extract_first_line(stderr);
        return InstallFailureKind::UnsupportedPlatform { detail };
    }
    // Exit code 2 is the install script's convention for unsupported
    // platform (see install_remote_server.sh).
    if exit_code == Some(2) && (lower.contains("unsupported") || lower.contains("uname")) {
        let detail = extract_first_line(stderr);
        return InstallFailureKind::UnsupportedPlatform { detail };
    }

    // --- Account issues ---
    if contains_any(
        &lower,
        &[
            "password has expired",
            "password expired",
            "your password has expired",
            "you are required to change your password",
            "you must change your password",
            "authentication token manipulation error",
            "pam_chauthtok",
        ],
    ) {
        return InstallFailureKind::AccountIssue {
            detail: "password expired".to_string(),
        };
    }
    if contains_any(
        &lower,
        &[
            "no tty present",
            "a]tty is required",
            "must be run from a terminal",
            "stdin is not a terminal",
            "stdin: is not a tty",
            "the input device is not a tty",
        ],
    ) {
        return InstallFailureKind::AccountIssue {
            detail: "no TTY available".to_string(),
        };
    }

    // --- Read-only filesystem (check before permission denied because
    //     some systems emit both) ---
    if contains_any(
        &lower,
        &["read-only file system", "read only file system", "erofs"],
    ) {
        return InstallFailureKind::ReadOnlyFilesystem;
    }

    // --- Permission denied ---
    if contains_any(
        &lower,
        &[
            "permission denied",
            "eacces",
            "operation not permitted",
            "cannot create directory",
        ],
    ) && !contains_any(&lower, &["publickey", "keyboard-interactive"])
    {
        return InstallFailureKind::PermissionDenied;
    }

    // --- Disk full / quota ---
    if contains_any(
        &lower,
        &[
            "no space left on device",
            "disk quota exceeded",
            "enospc",
            "edquot",
            "not enough space",
            "cannot allocate",
            "file too large",
        ],
    ) {
        return InstallFailureKind::DiskFull;
    }

    // --- Fallback ---
    InstallFailureKind::Unclassified {
        message: truncate_for_display(stderr, 256),
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Case-insensitive check: does `haystack` contain any of the `needles`?
fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|n| haystack.contains(n))
}

/// Extract a short detail string from SSH-related stderr. Takes the
/// first non-empty line (trimmed) or a generic fallback.
fn extract_ssh_detail(lower: &str) -> String {
    // Try to find the most informative line.
    for line in lower.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        // Skip generic SSH banner lines.
        if trimmed.starts_with("warning:") || trimmed.starts_with("debug") {
            continue;
        }
        return truncate_for_display(trimmed, 120);
    }
    "unknown SSH error".to_string()
}

/// Returns the first non-empty, trimmed line of `text`.
fn extract_first_line(text: &str) -> String {
    text.lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .map(|l| truncate_for_display(l, 120))
        .unwrap_or_default()
}

/// Truncates `s` to at most `max_len` bytes (on a char boundary),
/// appending "…" when truncated.
fn truncate_for_display(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        return s.to_string();
    }
    let mut end = max_len;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}…", &s[..end])
}

#[cfg(test)]
mod tests;
