/// Typed classification of remote-server install/setup failures.
///
/// Each variant maps to one or more families from the production error CSV.
/// The classifier preserves the raw stderr for diagnostics and telemetry,
/// but exposes a typed category with human-readable title and description
/// for the UI banner and telemetry event fields.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InstallFailureCategory {
    /// Install or download exceeded the configured timeout.
    Timeout,
    /// `curl` is not installed on the remote host (but `wget` may be).
    MissingCurl,
    /// `wget` is not installed on the remote host (but `curl` may be).
    MissingWget,
    /// Neither `curl` nor `wget` is available on the remote host.
    /// This is the category that triggers the SCP fallback.
    MissingHttpClient,
    /// `tar` is not installed on the remote host.
    MissingTar,
    /// `bash` is not available on the remote host (script interpreter).
    MissingBash,
    /// The remote host's CPU architecture is not supported.
    UnsupportedArchitecture,
    /// The remote host's OS is not supported.
    UnsupportedOs,
    /// DNS resolution failed for the download endpoint.
    DnsFailure,
    /// TCP connection to the download endpoint was refused.
    ConnectionRefused,
    /// The download endpoint is unreachable (network/routing failure).
    ConnectionUnreachable,
    /// TLS/SSL certificate verification failed.
    TlsCaFailure,
    /// HTTP 403 Forbidden from the download endpoint.
    HttpForbidden,
    /// HTTP 502 Bad Gateway from the download endpoint.
    HttpBadGateway,
    /// Other HTTP error from the download endpoint.
    HttpError,
    /// Download was interrupted or incomplete (partial transfer).
    PartialDownload,
    /// Could not write the downloaded file to disk.
    DownloadWriteFailure,
    /// Permission denied when creating the install directory or
    /// writing the binary.
    InstallDirPermissionDenied,
    /// No space left on the remote filesystem.
    NoSpaceLeft,
    /// The remote filesystem is mounted read-only.
    ReadOnlyFilesystem,
    /// `tar` extraction failed (corrupt archive, format error).
    TarExtractionFailure,
    /// `tar` extraction failed due to permission/ownership errors.
    TarPermissionFailure,
    /// The remote user's password has expired; SSH commands requiring
    /// a TTY prompt fail.
    ExpiredPassword,
    /// Permission denied when accessing shell startup files
    /// (e.g. `.bashrc`, `.profile`).
    StartupFilePermissionDenied,
    /// SSH connection was forcibly closed (exit code 255) or the
    /// remote end sent a disconnect.
    SshDisconnect,
    /// The install script exited with a non-zero code that doesn't
    /// match any recognized pattern.
    ScriptError,
    /// The failure could not be classified from the available stderr
    /// and exit code.
    Unknown,
}

impl InstallFailureCategory {
    /// Short, human-readable title for the UI banner header.
    pub fn title(&self) -> &'static str {
        match self {
            Self::Timeout => "Installation timed out",
            Self::MissingCurl => "curl is not installed",
            Self::MissingWget => "wget is not installed",
            Self::MissingHttpClient => "No HTTP client available",
            Self::MissingTar => "tar is not installed",
            Self::MissingBash => "bash is not available",
            Self::UnsupportedArchitecture => "Unsupported CPU architecture",
            Self::UnsupportedOs => "Unsupported operating system",
            Self::DnsFailure => "DNS resolution failed",
            Self::ConnectionRefused => "Connection refused",
            Self::ConnectionUnreachable => "Host unreachable",
            Self::TlsCaFailure => "TLS certificate error",
            Self::HttpForbidden => "Download forbidden (HTTP 403)",
            Self::HttpBadGateway => "Download server error (HTTP 502)",
            Self::HttpError => "Download failed (HTTP error)",
            Self::PartialDownload => "Download incomplete",
            Self::DownloadWriteFailure => "Could not save download",
            Self::InstallDirPermissionDenied => "Permission denied",
            Self::NoSpaceLeft => "No space left on device",
            Self::ReadOnlyFilesystem => "Read-only filesystem",
            Self::TarExtractionFailure => "Archive extraction failed",
            Self::TarPermissionFailure => "Extraction permission error",
            Self::ExpiredPassword => "Password expired",
            Self::StartupFilePermissionDenied => "Startup file permission error",
            Self::SshDisconnect => "SSH connection lost",
            Self::ScriptError => "Install script failed",
            Self::Unknown => "Installation failed",
        }
    }

    /// Longer description for the UI banner body text.
    pub fn description(&self) -> &'static str {
        match self {
            Self::Timeout => {
                "The installation timed out before completing. \
                 The remote host may have a slow network connection."
            }
            Self::MissingCurl => {
                "curl is not installed on the remote host. \
                 Install curl or ensure wget is available."
            }
            Self::MissingWget => {
                "wget is not installed on the remote host. \
                 Install wget or ensure curl is available."
            }
            Self::MissingHttpClient => {
                "Neither curl nor wget is available on the remote host. \
                 Install one of these HTTP clients to enable direct download."
            }
            Self::MissingTar => {
                "tar is not installed on the remote host. \
                 The archive cannot be extracted without tar."
            }
            Self::MissingBash => {
                "bash is not available on the remote host. \
                 The install script requires bash to run."
            }
            Self::UnsupportedArchitecture => {
                "The remote host's CPU architecture is not supported by \
                 the prebuilt binary."
            }
            Self::UnsupportedOs => {
                "The remote host's operating system is not supported by \
                 the prebuilt binary."
            }
            Self::DnsFailure => {
                "Could not resolve the download server's hostname. \
                 Check the remote host's DNS configuration."
            }
            Self::ConnectionRefused => {
                "The download server actively refused the connection. \
                 A firewall or proxy may be blocking outbound HTTPS."
            }
            Self::ConnectionUnreachable => {
                "The download server is unreachable from the remote host. \
                 Check the network configuration and firewall rules."
            }
            Self::TlsCaFailure => {
                "TLS certificate verification failed. The remote host's \
                 CA certificates may be missing or outdated."
            }
            Self::HttpForbidden => {
                "The download server returned HTTP 403 Forbidden. \
                 The download URL may have expired or be restricted."
            }
            Self::HttpBadGateway => {
                "The download server returned HTTP 502 Bad Gateway. \
                 This is usually a transient server-side issue."
            }
            Self::HttpError => {
                "The download failed with an HTTP error. \
                 The server may be temporarily unavailable."
            }
            Self::PartialDownload => {
                "The download was interrupted before completing. \
                 The network connection may be unstable."
            }
            Self::DownloadWriteFailure => {
                "Could not write the downloaded file to disk. \
                 Check disk space and directory permissions."
            }
            Self::InstallDirPermissionDenied => {
                "Permission denied when writing to the install directory. \
                 Check that the remote user has write access to the home directory."
            }
            Self::NoSpaceLeft => {
                "The remote filesystem has no space left. \
                 Free up disk space and try again."
            }
            Self::ReadOnlyFilesystem => {
                "The remote filesystem is mounted read-only. \
                 The binary cannot be installed on a read-only filesystem."
            }
            Self::TarExtractionFailure => {
                "The downloaded archive could not be extracted. \
                 The file may be corrupt or in an unexpected format."
            }
            Self::TarPermissionFailure => {
                "Archive extraction failed due to a permission error. \
                 tar could not set file ownership or permissions."
            }
            Self::ExpiredPassword => {
                "The remote user's password has expired. \
                 Update the password on the remote host and reconnect."
            }
            Self::StartupFilePermissionDenied => {
                "A shell startup file (e.g. .bashrc, .profile) could not be read. \
                 Check file permissions on the remote host."
            }
            Self::SshDisconnect => {
                "The SSH connection was lost during installation. \
                 The remote host may have disconnected or the network dropped."
            }
            Self::ScriptError => {
                "The install script exited with an error. \
                 Check the error details below for more information."
            }
            Self::Unknown => {
                "The installation failed for an unknown reason. \
                 Check the error details below for more information."
            }
        }
    }

    /// Stable snake_case identifier for telemetry and serialization.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Timeout => "timeout",
            Self::MissingCurl => "missing_curl",
            Self::MissingWget => "missing_wget",
            Self::MissingHttpClient => "missing_http_client",
            Self::MissingTar => "missing_tar",
            Self::MissingBash => "missing_bash",
            Self::UnsupportedArchitecture => "unsupported_architecture",
            Self::UnsupportedOs => "unsupported_os",
            Self::DnsFailure => "dns_failure",
            Self::ConnectionRefused => "connection_refused",
            Self::ConnectionUnreachable => "connection_unreachable",
            Self::TlsCaFailure => "tls_ca_failure",
            Self::HttpForbidden => "http_forbidden",
            Self::HttpBadGateway => "http_bad_gateway",
            Self::HttpError => "http_error",
            Self::PartialDownload => "partial_download",
            Self::DownloadWriteFailure => "download_write_failure",
            Self::InstallDirPermissionDenied => "install_dir_permission_denied",
            Self::NoSpaceLeft => "no_space_left",
            Self::ReadOnlyFilesystem => "read_only_filesystem",
            Self::TarExtractionFailure => "tar_extraction_failure",
            Self::TarPermissionFailure => "tar_permission_failure",
            Self::ExpiredPassword => "expired_password",
            Self::StartupFilePermissionDenied => "startup_file_permission_denied",
            Self::SshDisconnect => "ssh_disconnect",
            Self::ScriptError => "script_error",
            Self::Unknown => "unknown",
        }
    }
}

impl std::fmt::Display for InstallFailureCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Classifies a raw install/setup failure into a typed
/// [`InstallFailureCategory`].
///
/// The classifier inspects both the stderr text and the process exit code
/// to determine the most specific category. Pattern matching is done
/// case-insensitively against known error signatures from `curl`, `wget`,
/// `tar`, `bash`, `ssh`, and common shell/OS error messages.
///
/// When multiple patterns match, the first (most specific) match wins.
pub fn classify_install_failure(
    stderr: &str,
    exit_code: Option<i32>,
) -> InstallFailureCategory {
    let lower = stderr.to_ascii_lowercase();

    // --- SSH-level failures (exit 255 or explicit disconnect) ---
    if exit_code == Some(255) {
        return InstallFailureCategory::SshDisconnect;
    }
    if lower.contains("connection reset by peer")
        || lower.contains("broken pipe")
        || lower.contains("ssh_exchange_identification")
        || lower.contains("packet_write_wait")
        || lower.contains("connection closed by remote host")
        || lower.contains("client_loop: send disconnect")
    {
        return InstallFailureCategory::SshDisconnect;
    }

    // --- Timeout ---
    if lower.contains("timed out")
        || lower.contains("timeout")
        || lower.contains("operation timed out")
        || lower.contains("connection timed out")
    {
        return InstallFailureCategory::Timeout;
    }

    // --- Password/auth failures ---
    if lower.contains("password has expired")
        || lower.contains("password expired")
        || lower.contains("your password has expired")
        || lower.contains("you are required to change your password")
    {
        return InstallFailureCategory::ExpiredPassword;
    }

    // --- Missing tools (exit code 127 = command not found) ---
    if lower.contains("neither curl nor wget")
        || lower.contains("error: neither curl nor wget is available")
    {
        return InstallFailureCategory::MissingHttpClient;
    }
    // The install script uses a specific exit code for no HTTP client.
    if exit_code == Some(super::NO_HTTP_CLIENT_EXIT_CODE) {
        return InstallFailureCategory::MissingHttpClient;
    }
    if (lower.contains("curl") || lower.contains("curl:"))
        && (lower.contains("not found") || lower.contains("command not found"))
    {
        return InstallFailureCategory::MissingCurl;
    }
    if (lower.contains("wget") || lower.contains("wget:"))
        && (lower.contains("not found") || lower.contains("command not found"))
    {
        return InstallFailureCategory::MissingWget;
    }
    if lower.contains("tar:") && lower.contains("not found")
        || lower.contains("tar: command not found")
    {
        return InstallFailureCategory::MissingTar;
    }
    if lower.contains("bash") && lower.contains("not found")
        || lower.contains("bash: command not found")
        || lower.contains("bash: no such file or directory")
    {
        return InstallFailureCategory::MissingBash;
    }

    // --- Unsupported arch/OS (from the install script's stderr) ---
    if lower.contains("unsupported arch") {
        return InstallFailureCategory::UnsupportedArchitecture;
    }
    if lower.contains("unsupported os") {
        return InstallFailureCategory::UnsupportedOs;
    }

    // --- DNS failure ---
    if lower.contains("could not resolve host")
        || lower.contains("could not resolve")
        || lower.contains("name or service not known")
        || lower.contains("temporary failure in name resolution")
        || lower.contains("unable to resolve host")
        || lower.contains("dns_error")
    {
        return InstallFailureCategory::DnsFailure;
    }

    // --- Connection refused ---
    if lower.contains("connection refused") {
        return InstallFailureCategory::ConnectionRefused;
    }

    // --- Connection unreachable ---
    if lower.contains("no route to host")
        || lower.contains("network is unreachable")
        || lower.contains("host is unreachable")
        || lower.contains("network unreachable")
    {
        return InstallFailureCategory::ConnectionUnreachable;
    }

    // --- TLS/CA failures ---
    if lower.contains("ssl certificate problem")
        || lower.contains("certificate verify failed")
        || lower.contains("ssl_error")
        || lower.contains("unable to get local issuer certificate")
        || lower.contains("ca certificate")
        || lower.contains("self signed certificate")
        || lower.contains("certificate is not trusted")
        || lower.contains("unable to locally verify")
        || lower.contains("ssl handshake")
    {
        return InstallFailureCategory::TlsCaFailure;
    }

    // --- HTTP status codes ---
    // curl with -f emits "The requested URL returned error: 403" or
    // "curl: (22) The requested URL returned error: 403 Forbidden"
    if lower.contains("403 forbidden") || lower.contains("returned error: 403") {
        return InstallFailureCategory::HttpForbidden;
    }
    if lower.contains("502 bad gateway") || lower.contains("returned error: 502") {
        return InstallFailureCategory::HttpBadGateway;
    }
    // Generic HTTP error from curl -f (exit code 22)
    if lower.contains("the requested url returned error")
        || (exit_code == Some(22) && lower.contains("curl"))
    {
        return InstallFailureCategory::HttpError;
    }
    // wget HTTP errors
    if lower.contains("error 4") && lower.contains("wget") {
        return InstallFailureCategory::HttpError;
    }

    // --- Partial download ---
    if lower.contains("partial file")
        || lower.contains("transfer closed with")
        || lower.contains("curl: (18)")
        || lower.contains("connection was reset")
        || lower.contains("incomplete download")
    {
        return InstallFailureCategory::PartialDownload;
    }

    // --- Read-only filesystem (check before general permission denied) ---
    if lower.contains("read-only file system") || lower.contains("erofs") {
        return InstallFailureCategory::ReadOnlyFilesystem;
    }

    // --- No space left ---
    if lower.contains("no space left on device")
        || lower.contains("disk quota exceeded")
        || lower.contains("enospc")
    {
        return InstallFailureCategory::NoSpaceLeft;
    }

    // --- Download write failure ---
    // Exclude lines that are clearly tar errors ("tar:" prefix), but
    // allow filenames like "oz.tar.gz" in curl error messages.
    if (lower.contains("failed writing body")
        || lower.contains("write error")
        || lower.contains("failed to create file")
        || lower.contains("curl: (23)"))
        && !lower.contains("tar:")
    {
        return InstallFailureCategory::DownloadWriteFailure;
    }

    // --- Tar permission/ownership failure ---
    if lower.contains("tar:")
        && (lower.contains("cannot change ownership")
            || lower.contains("operation not permitted")
            || lower.contains("cannot open: permission denied"))
    {
        return InstallFailureCategory::TarPermissionFailure;
    }

    // --- Tar extraction failure ---
    if lower.contains("tar:")
        && (lower.contains("error is not recoverable")
            || lower.contains("not in gzip format")
            || lower.contains("unexpected eof")
            || lower.contains("invalid tar")
            || lower.contains("damaged")
            || lower.contains("truncated")
            || lower.contains("corrupted"))
    {
        return InstallFailureCategory::TarExtractionFailure;
    }
    // Generic tar failure
    if lower.contains("tar:") && lower.contains("exiting with failure") {
        return InstallFailureCategory::TarExtractionFailure;
    }

    // --- Install dir permission denied ---
    if lower.contains("permission denied") && lower.contains("mkdir") {
        return InstallFailureCategory::InstallDirPermissionDenied;
    }
    if lower.contains("permission denied") && lower.contains("mv ") {
        return InstallFailureCategory::InstallDirPermissionDenied;
    }
    if lower.contains("permission denied") && lower.contains("chmod") {
        return InstallFailureCategory::InstallDirPermissionDenied;
    }

    // --- Startup file permission denied ---
    if lower.contains("permission denied")
        && (lower.contains(".bashrc")
            || lower.contains(".bash_profile")
            || lower.contains(".profile")
            || lower.contains(".zshrc")
            || lower.contains(".zprofile")
            || lower.contains("startup"))
    {
        return InstallFailureCategory::StartupFilePermissionDenied;
    }

    // --- Generic permission denied (after more specific checks) ---
    if lower.contains("permission denied") {
        return InstallFailureCategory::InstallDirPermissionDenied;
    }

    // --- Script error (non-zero exit but no matching pattern) ---
    if let Some(code) = exit_code {
        if code != 0 {
            return InstallFailureCategory::ScriptError;
        }
    }

    InstallFailureCategory::Unknown
}

/// A classified install failure bundling the typed category with the
/// raw stderr for diagnostics.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClassifiedInstallFailure {
    /// The typed failure category.
    pub category: InstallFailureCategory,
    /// The original raw stderr text, preserved for diagnostics and
    /// detailed error display.
    pub raw_stderr: String,
    /// The original process exit code, if available.
    pub exit_code: Option<i32>,
}

impl ClassifiedInstallFailure {
    /// Classify a raw failure.
    pub fn from_raw(stderr: &str, exit_code: Option<i32>) -> Self {
        let category = classify_install_failure(stderr, exit_code);
        Self {
            category,
            raw_stderr: stderr.to_owned(),
            exit_code,
        }
    }

    /// Classify from a combined error string like
    /// `"install script failed (exit 1): <stderr>"`.
    pub fn from_error_string(error: &str) -> Self {
        let (exit_code, stderr) = parse_error_string(error);
        Self::from_raw(stderr, exit_code)
    }
}

/// Parses an error string of the form
/// `"install script failed (exit <code>): <stderr>"` or
/// `"<anything> (exit <code>): <stderr>"` into its exit code and stderr
/// components. Falls back to treating the whole string as stderr with
/// no exit code.
fn parse_error_string(error: &str) -> (Option<i32>, &str) {
    // Try to find "(exit <N>): " pattern.
    if let Some(exit_start) = error.find("(exit ") {
        let after_exit = &error[exit_start + 6..];
        if let Some(paren_end) = after_exit.find(')') {
            let code_str = &after_exit[..paren_end];
            let exit_code = code_str.trim().parse::<i32>().ok();
            // The stderr follows "): "
            let stderr_start = exit_start + 6 + paren_end + 1;
            let stderr = if stderr_start < error.len() {
                error[stderr_start..].trim_start_matches(": ").trim()
            } else {
                ""
            };
            return (exit_code, stderr);
        }
    }
    (None, error)
}

#[cfg(test)]
#[path = "install_failure_tests.rs"]
mod tests;
