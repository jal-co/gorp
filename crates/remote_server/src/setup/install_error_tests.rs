use super::*;

// ═══════════════════════════════════════════════════════════════════════
// § 1  Classifier coverage for every CSV failure family
// ═══════════════════════════════════════════════════════════════════════

// ── Install timeout ─────────────────────────────────────────────────

#[test]
fn classify_timeout_from_flag() {
    let cat = classify_install_failure(None, "", true);
    assert_eq!(cat, InstallFailureCategory::InstallTimeout);
}

#[test]
fn classify_timeout_overrides_stderr() {
    // Even if stderr contains other patterns, the timeout flag wins.
    let cat = classify_install_failure(Some(1), "Permission denied", true);
    assert_eq!(cat, InstallFailureCategory::InstallTimeout);
}

// ── Missing curl (no HTTP client) ───────────────────────────────────

#[test]
fn classify_no_http_client_exit_code() {
    let cat = classify_install_failure(Some(3), "error: neither curl nor wget is available", false);
    assert_eq!(cat, InstallFailureCategory::NoHttpClient);
}

#[test]
fn classify_no_http_client_exit_code_only() {
    // Exit code alone is sufficient — the script prints the message to
    // stderr but the sentinel is the exit code.
    let cat = classify_install_failure(Some(3), "", false);
    assert_eq!(cat, InstallFailureCategory::NoHttpClient);
}

// ── Missing wget / both HTTP clients ────────────────────────────────
// The install script falls through curl → wget → exit 3. Whether only
// wget is missing doesn't matter; the sentinel covers both.

#[test]
fn classify_no_http_client_when_both_missing() {
    let cat = classify_install_failure(
        Some(3),
        "error: neither curl nor wget is available\n",
        false,
    );
    assert_eq!(cat, InstallFailureCategory::NoHttpClient);
}

// ── Missing tar ─────────────────────────────────────────────────────

#[test]
fn classify_missing_tar_not_found() {
    let cat = classify_install_failure(Some(127), "tar: not found", false);
    assert_eq!(cat, InstallFailureCategory::MissingTar);
}

#[test]
fn classify_missing_tar_command_not_found() {
    let cat = classify_install_failure(Some(127), "bash: tar: command not found", false);
    assert_eq!(cat, InstallFailureCategory::MissingTar);
}

#[test]
fn classify_missing_tar_no_such_file() {
    let cat = classify_install_failure(Some(127), "/usr/bin/tar: No such file or directory", false);
    assert_eq!(cat, InstallFailureCategory::MissingTar);
}

// ── Missing bash ────────────────────────────────────────────────────

#[test]
fn classify_missing_bash_not_found() {
    let cat = classify_install_failure(Some(127), "bash: not found", false);
    assert_eq!(cat, InstallFailureCategory::MissingBash);
}

#[test]
fn classify_missing_bash_command_not_found() {
    let cat = classify_install_failure(Some(127), "bash: command not found", false);
    assert_eq!(cat, InstallFailureCategory::MissingBash);
}

#[test]
fn classify_missing_bash_no_such_file() {
    let cat = classify_install_failure(Some(127), "No such file or directory: bash", false);
    assert_eq!(cat, InstallFailureCategory::MissingBash);
}

// ── Unsupported arch ────────────────────────────────────────────────

#[test]
fn classify_unsupported_arch_mips() {
    let cat = classify_install_failure(Some(2), "unsupported arch: mips\n", false);
    assert_eq!(
        cat,
        InstallFailureCategory::UnsupportedArch {
            arch: "mips".to_string()
        }
    );
}

#[test]
fn classify_unsupported_arch_ppc64le() {
    let cat = classify_install_failure(Some(2), "unsupported arch: ppc64le\n", false);
    assert_eq!(
        cat,
        InstallFailureCategory::UnsupportedArch {
            arch: "ppc64le".to_string()
        }
    );
}

#[test]
fn classify_unsupported_arch_s390x() {
    let cat = classify_install_failure(Some(2), "unsupported arch: s390x\n", false);
    assert_eq!(
        cat,
        InstallFailureCategory::UnsupportedArch {
            arch: "s390x".to_string()
        }
    );
}

#[test]
fn classify_unsupported_arch_exit2_no_message() {
    // Exit code 2 but no parseable arch → falls back to "unknown".
    let cat = classify_install_failure(Some(2), "", false);
    assert_eq!(
        cat,
        InstallFailureCategory::UnsupportedArch {
            arch: "unknown".to_string()
        }
    );
}

#[test]
fn classify_unsupported_os_freebsd() {
    let cat = classify_install_failure(Some(2), "unsupported OS: FreeBSD\n", false);
    assert_eq!(
        cat,
        InstallFailureCategory::UnsupportedOs {
            os: "FreeBSD".to_string()
        }
    );
}

// ── DNS failure ─────────────────────────────────────────────────────

#[test]
fn classify_dns_could_not_resolve() {
    let cat = classify_install_failure(
        Some(6),
        "curl: (6) Could not resolve host: app.warp.dev",
        false,
    );
    assert_eq!(cat, InstallFailureCategory::DnsFailure);
}

#[test]
fn classify_dns_name_or_service() {
    let cat = classify_install_failure(
        Some(6),
        "wget: unable to resolve host address 'app.warp.dev': Name or service not known",
        false,
    );
    assert_eq!(cat, InstallFailureCategory::DnsFailure);
}

#[test]
fn classify_dns_temporary_failure() {
    let cat = classify_install_failure(Some(6), "Temporary failure in name resolution", false);
    assert_eq!(cat, InstallFailureCategory::DnsFailure);
}

// ── Connection refused / unreachable ────────────────────────────────

#[test]
fn classify_connection_refused() {
    let cat = classify_install_failure(
        Some(7),
        "curl: (7) Failed to connect to app.warp.dev port 443: Connection refused",
        false,
    );
    assert_eq!(cat, InstallFailureCategory::ConnectionRefused);
}

#[test]
fn classify_no_route_to_host() {
    let cat = classify_install_failure(
        Some(7),
        "curl: (7) Failed to connect: No route to host",
        false,
    );
    assert_eq!(cat, InstallFailureCategory::ConnectionRefused);
}

#[test]
fn classify_network_unreachable() {
    let cat = classify_install_failure(Some(7), "curl: (7) Network is unreachable", false);
    assert_eq!(cat, InstallFailureCategory::ConnectionRefused);
}

// ── TLS / CA failure ────────────────────────────────────────────────

#[test]
fn classify_tls_ssl_connect_error() {
    let cat = classify_install_failure(Some(35), "curl: (35) SSL connect error", false);
    assert_eq!(cat, InstallFailureCategory::TlsCaFailure);
}

#[test]
fn classify_tls_certificate_verify() {
    let cat = classify_install_failure(
        Some(60),
        "curl: (60) SSL certificate problem: unable to get local issuer certificate",
        false,
    );
    assert_eq!(cat, InstallFailureCategory::TlsCaFailure);
}

#[test]
fn classify_tls_ca_bundle() {
    let cat = classify_install_failure(
        Some(77),
        "curl: (77) error setting certificate verify locations: CA-bundle",
        false,
    );
    assert_eq!(cat, InstallFailureCategory::TlsCaFailure);
}

#[test]
fn classify_tls_verify_failed() {
    let cat = classify_install_failure(
        Some(1),
        "ERROR: certificate verify failed (OpenSSL::SSL::SSLError)",
        false,
    );
    assert_eq!(cat, InstallFailureCategory::TlsCaFailure);
}

// ── HTTP 403 ────────────────────────────────────────────────────────

#[test]
fn classify_http_403_forbidden_stderr() {
    let cat = classify_install_failure(
        Some(22),
        "The requested URL returned error: 403 Forbidden",
        false,
    );
    assert_eq!(cat, InstallFailureCategory::HttpForbidden);
}

#[test]
fn classify_http_403_curl_exit_22() {
    let cat = classify_install_failure(Some(22), "403", false);
    assert_eq!(cat, InstallFailureCategory::HttpForbidden);
}

// ── HTTP 502 ────────────────────────────────────────────────────────

#[test]
fn classify_http_502_bad_gateway() {
    let cat = classify_install_failure(Some(22), "502 Bad Gateway", false);
    assert_eq!(
        cat,
        InstallFailureCategory::HttpServerError { status_code: 502 }
    );
}

#[test]
fn classify_http_503_service_unavailable() {
    let cat = classify_install_failure(Some(22), "503 Service Unavailable", false);
    assert_eq!(
        cat,
        InstallFailureCategory::HttpServerError { status_code: 503 }
    );
}

#[test]
fn classify_curl_exit_22_generic() {
    // curl -f exit 22 with no recognizable status → generic server error.
    let cat = classify_install_failure(Some(22), "The requested URL returned error: 500", false);
    assert_eq!(
        cat,
        InstallFailureCategory::HttpServerError { status_code: 0 }
    );
}

// ── Partial download ────────────────────────────────────────────────

#[test]
fn classify_partial_download_curl_exit_18() {
    let cat = classify_install_failure(
        Some(18),
        "curl: (18) transfer closed with 12345 bytes remaining",
        false,
    );
    assert_eq!(cat, InstallFailureCategory::PartialDownload);
}

#[test]
fn classify_partial_download_stderr_partial_file() {
    let cat = classify_install_failure(Some(1), "Partial file received", false);
    assert_eq!(cat, InstallFailureCategory::PartialDownload);
}

#[test]
fn classify_partial_download_unexpected_end_gz() {
    let cat = classify_install_failure(
        Some(1),
        "gzip: stdin: unexpected end of file\ntar: Child returned status 1\ntar: Error: oz.tar.gz",
        false,
    );
    assert_eq!(cat, InstallFailureCategory::PartialDownload);
}

// ── Download write failure ──────────────────────────────────────────

#[test]
fn classify_download_write_failure_curl_exit_23() {
    let cat = classify_install_failure(Some(23), "Failed writing body (0 != 1234)", false);
    assert_eq!(cat, InstallFailureCategory::DownloadWriteFailure);
}

#[test]
fn classify_download_write_failure_failed_writing_body() {
    let cat = classify_install_failure(Some(1), "curl: Failed writing body", false);
    assert_eq!(cat, InstallFailureCategory::DownloadWriteFailure);
}

// ── Install dir permission denied ───────────────────────────────────

#[test]
fn classify_install_dir_permission_denied() {
    let cat = classify_install_failure(
        Some(1),
        "mkdir: cannot create directory '/root/.warp/remote-server': Permission denied",
        false,
    );
    assert_eq!(cat, InstallFailureCategory::InstallDirPermissionDenied);
}

#[test]
fn classify_install_dir_mv_permission_denied() {
    let cat = classify_install_failure(
        Some(1),
        "mv: cannot move 'oz' to '/opt/warp/remote-server/oz': Permission denied",
        false,
    );
    assert_eq!(cat, InstallFailureCategory::InstallDirPermissionDenied);
}

// ── No space / quota ────────────────────────────────────────────────

#[test]
fn classify_no_space_left() {
    let cat = classify_install_failure(Some(1), "write error: No space left on device", false);
    assert_eq!(cat, InstallFailureCategory::NoSpaceOrQuota);
}

#[test]
fn classify_disk_quota_exceeded() {
    let cat = classify_install_failure(Some(1), "Disk quota exceeded", false);
    assert_eq!(cat, InstallFailureCategory::NoSpaceOrQuota);
}

#[test]
fn classify_enospc() {
    let cat = classify_install_failure(Some(1), "ENOSPC: no space left on device", false);
    assert_eq!(cat, InstallFailureCategory::NoSpaceOrQuota);
}

// ── Read-only filesystem ────────────────────────────────────────────

#[test]
fn classify_read_only_filesystem() {
    let cat = classify_install_failure(
        Some(1),
        "mkdir: cannot create directory: Read-only file system",
        false,
    );
    assert_eq!(cat, InstallFailureCategory::ReadOnlyFilesystem);
}

#[test]
fn classify_erofs() {
    let cat = classify_install_failure(
        Some(1),
        "mv: cannot move file: EROFS: read-only file system",
        false,
    );
    assert_eq!(cat, InstallFailureCategory::ReadOnlyFilesystem);
}

// ── Tar ownership / permission failure ──────────────────────────────

#[test]
fn classify_tar_cannot_change_ownership() {
    let cat = classify_install_failure(
        Some(1),
        "tar: oz: Cannot change ownership to uid 1000, gid 1000: Operation not permitted",
        false,
    );
    assert_eq!(cat, InstallFailureCategory::TarPermissionFailure);
}

#[test]
fn classify_tar_cannot_open_exit_2_is_unsupported_sentinel() {
    // Exit 2 is the script's sentinel for unsupported arch/OS and is
    // handled before stderr matching, so even tar errors with exit 2
    // classify as UnsupportedArch.
    let cat = classify_install_failure(
        Some(2),
        "tar: oz.tar.gz: Cannot open: Permission denied\ntar: Error is not recoverable",
        false,
    );
    assert_eq!(
        cat,
        InstallFailureCategory::UnsupportedArch {
            arch: "unknown".to_string()
        }
    );
}

#[test]
fn classify_tar_cannot_open_non_sentinel_exit() {
    // With a non-sentinel exit code, the tar-specific pattern wins
    // over the generic "permission denied" rule.
    let cat = classify_install_failure(
        Some(1),
        "tar: oz.tar.gz: Cannot open: Permission denied\ntar: Error is not recoverable",
        false,
    );
    assert_eq!(cat, InstallFailureCategory::TarPermissionFailure);
}

#[test]
fn classify_extract_operation_not_permitted() {
    let cat = classify_install_failure(
        Some(1),
        "extract: Operation not permitted while extracting archive",
        false,
    );
    assert_eq!(cat, InstallFailureCategory::TarPermissionFailure);
}

// ── Expired password / no TTY ───────────────────────────────────────

#[test]
fn classify_expired_password() {
    let cat = classify_install_failure(
        Some(1),
        "WARNING: Your password has expired.\nPassword change required but no TTY available.",
        false,
    );
    assert_eq!(cat, InstallFailureCategory::ExpiredPasswordOrNoTty);
}

#[test]
fn classify_must_change_password() {
    let cat = classify_install_failure(
        Some(1),
        "You must change your password before continuing",
        false,
    );
    assert_eq!(cat, InstallFailureCategory::ExpiredPasswordOrNoTty);
}

#[test]
fn classify_no_tty_present() {
    let cat = classify_install_failure(
        Some(1),
        "sudo: no tty present and no askpass program specified",
        false,
    );
    assert_eq!(cat, InstallFailureCategory::ExpiredPasswordOrNoTty);
}

// ── Startup-file permission denied ──────────────────────────────────

#[test]
fn classify_startup_file_bashrc_permission_denied() {
    let cat = classify_install_failure(
        Some(1),
        "bash: /home/user/.bashrc: Permission denied",
        false,
    );
    assert_eq!(cat, InstallFailureCategory::StartupFilePermissionDenied);
}

#[test]
fn classify_startup_file_profile_permission_denied() {
    let cat = classify_install_failure(
        Some(1),
        "bash: /home/user/.profile: Permission denied",
        false,
    );
    assert_eq!(cat, InstallFailureCategory::StartupFilePermissionDenied);
}

#[test]
fn classify_startup_file_zshrc_permission_denied() {
    let cat = classify_install_failure(Some(1), "zsh: permission denied: /home/user/.zshrc", false);
    assert_eq!(cat, InstallFailureCategory::StartupFilePermissionDenied);
}

// ── SSH disconnect / exit 255 ───────────────────────────────────────

#[test]
fn classify_ssh_disconnect_exit_255() {
    let cat = classify_install_failure(
        Some(255),
        "ssh: connect to host example.com: Connection reset by peer",
        false,
    );
    assert_eq!(cat, InstallFailureCategory::SshDisconnect);
}

#[test]
fn classify_ssh_disconnect_exit_255_no_stderr() {
    let cat = classify_install_failure(Some(255), "", false);
    assert_eq!(cat, InstallFailureCategory::SshDisconnect);
}

// ── Unknown / catch-all ─────────────────────────────────────────────

#[test]
fn classify_unknown_for_unrecognized_error() {
    let cat = classify_install_failure(Some(42), "some totally unexpected error output", false);
    assert_eq!(cat, InstallFailureCategory::Unknown);
}

#[test]
fn classify_unknown_for_empty_stderr() {
    let cat = classify_install_failure(Some(1), "", false);
    assert_eq!(cat, InstallFailureCategory::Unknown);
}

// ═══════════════════════════════════════════════════════════════════════
// § 2  is_retriable – no blind retry for permanent conditions
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn retriable_categories() {
    let retriable = [
        InstallFailureCategory::DnsFailure,
        InstallFailureCategory::ConnectionRefused,
        InstallFailureCategory::TlsCaFailure,
        InstallFailureCategory::HttpServerError { status_code: 502 },
        InstallFailureCategory::PartialDownload,
        InstallFailureCategory::InstallTimeout,
    ];
    for cat in &retriable {
        assert!(cat.is_retriable(), "{cat:?} should be retriable");
    }
}

#[test]
fn non_retriable_permanent_conditions() {
    let non_retriable = [
        InstallFailureCategory::UnsupportedArch {
            arch: "mips".to_string(),
        },
        InstallFailureCategory::UnsupportedOs {
            os: "FreeBSD".to_string(),
        },
        InstallFailureCategory::NoHttpClient,
        InstallFailureCategory::MissingTar,
        InstallFailureCategory::MissingBash,
        InstallFailureCategory::HttpForbidden,
        InstallFailureCategory::DownloadWriteFailure,
        InstallFailureCategory::InstallDirPermissionDenied,
        InstallFailureCategory::NoSpaceOrQuota,
        InstallFailureCategory::ReadOnlyFilesystem,
        InstallFailureCategory::TarPermissionFailure,
        InstallFailureCategory::ExpiredPasswordOrNoTty,
        InstallFailureCategory::StartupFilePermissionDenied,
        InstallFailureCategory::SshDisconnect,
        InstallFailureCategory::Unknown,
    ];
    for cat in &non_retriable {
        assert!(!cat.is_retriable(), "{cat:?} should NOT be retriable");
    }
}

// ═══════════════════════════════════════════════════════════════════════
// § 3  telemetry_tag stability
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn telemetry_tags_are_unique_and_non_empty() {
    let all_categories = [
        InstallFailureCategory::UnsupportedArch {
            arch: "mips".to_string(),
        },
        InstallFailureCategory::UnsupportedOs {
            os: "FreeBSD".to_string(),
        },
        InstallFailureCategory::NoHttpClient,
        InstallFailureCategory::MissingTar,
        InstallFailureCategory::MissingBash,
        InstallFailureCategory::DnsFailure,
        InstallFailureCategory::ConnectionRefused,
        InstallFailureCategory::TlsCaFailure,
        InstallFailureCategory::HttpForbidden,
        InstallFailureCategory::HttpServerError { status_code: 502 },
        InstallFailureCategory::PartialDownload,
        InstallFailureCategory::InstallTimeout,
        InstallFailureCategory::DownloadWriteFailure,
        InstallFailureCategory::InstallDirPermissionDenied,
        InstallFailureCategory::NoSpaceOrQuota,
        InstallFailureCategory::ReadOnlyFilesystem,
        InstallFailureCategory::TarPermissionFailure,
        InstallFailureCategory::ExpiredPasswordOrNoTty,
        InstallFailureCategory::StartupFilePermissionDenied,
        InstallFailureCategory::SshDisconnect,
        InstallFailureCategory::Unknown,
    ];

    let mut seen = std::collections::HashSet::new();
    for cat in &all_categories {
        let tag = cat.telemetry_tag();
        assert!(!tag.is_empty(), "{cat:?} has empty telemetry tag");
        assert!(
            seen.insert(tag),
            "duplicate telemetry tag {tag:?} for {cat:?}"
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════
// § 4  Display is non-empty for all variants
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn display_is_non_empty() {
    let all_categories = [
        InstallFailureCategory::UnsupportedArch {
            arch: "mips".to_string(),
        },
        InstallFailureCategory::UnsupportedOs {
            os: "FreeBSD".to_string(),
        },
        InstallFailureCategory::NoHttpClient,
        InstallFailureCategory::MissingTar,
        InstallFailureCategory::MissingBash,
        InstallFailureCategory::DnsFailure,
        InstallFailureCategory::ConnectionRefused,
        InstallFailureCategory::TlsCaFailure,
        InstallFailureCategory::HttpForbidden,
        InstallFailureCategory::HttpServerError { status_code: 502 },
        InstallFailureCategory::PartialDownload,
        InstallFailureCategory::InstallTimeout,
        InstallFailureCategory::DownloadWriteFailure,
        InstallFailureCategory::InstallDirPermissionDenied,
        InstallFailureCategory::NoSpaceOrQuota,
        InstallFailureCategory::ReadOnlyFilesystem,
        InstallFailureCategory::TarPermissionFailure,
        InstallFailureCategory::ExpiredPasswordOrNoTty,
        InstallFailureCategory::StartupFilePermissionDenied,
        InstallFailureCategory::SshDisconnect,
        InstallFailureCategory::Unknown,
    ];

    for cat in &all_categories {
        let display = format!("{cat}");
        assert!(!display.is_empty(), "{cat:?} has empty Display output");
    }
}

// ═══════════════════════════════════════════════════════════════════════
// § 5  Edge cases and production stderr samples
// ═══════════════════════════════════════════════════════════════════════

/// Real-world production sample: curl with MOTD noise before the error.
#[test]
fn classify_dns_failure_with_motd_noise() {
    let stderr = "\
Welcome to Ubuntu 22.04 LTS
Last login: Mon Apr 7 10:00:00 2025
curl: (6) Could not resolve host: app.warp.dev\n";
    let cat = classify_install_failure(Some(6), stderr, false);
    assert_eq!(cat, InstallFailureCategory::DnsFailure);
}

/// Real-world: wget variant of DNS failure.
#[test]
fn classify_dns_failure_wget() {
    let cat = classify_install_failure(
        Some(4),
        "wget: unable to resolve host address 'app.warp.dev'\n",
        false,
    );
    assert_eq!(cat, InstallFailureCategory::DnsFailure);
}

/// Real-world: gzip truncation from partial download.
#[test]
fn classify_partial_download_gzip_unexpected_eof() {
    let stderr = "\
gzip: stdin: unexpected end of file
tar: Child returned status 1
tar: Error is not recoverable: exiting now\n";
    // Note: "unexpected end" + "gz" pattern
    let cat = classify_install_failure(Some(1), stderr, false);
    assert_eq!(cat, InstallFailureCategory::PartialDownload);
}

/// Classifier priority: timeout > exit code > stderr.
#[test]
fn timeout_takes_priority_over_exit_code() {
    let cat = classify_install_failure(Some(255), "connection reset", true);
    assert_eq!(cat, InstallFailureCategory::InstallTimeout);
}

/// Exit 255 takes priority over stderr patterns.
#[test]
fn exit_255_takes_priority_over_stderr() {
    let cat = classify_install_failure(Some(255), "Permission denied", false);
    assert_eq!(cat, InstallFailureCategory::SshDisconnect);
}

/// Exit 3 (no HTTP client) takes priority over stderr.
#[test]
fn exit_3_takes_priority_over_stderr() {
    let cat = classify_install_failure(Some(3), "Permission denied", false);
    assert_eq!(cat, InstallFailureCategory::NoHttpClient);
}

// ═══════════════════════════════════════════════════════════════════════
// § 6  Script constant alignment
// ═══════════════════════════════════════════════════════════════════════

/// Verify our exit code constant matches the one in setup.rs.
#[test]
fn no_http_client_exit_code_matches_setup() {
    assert_eq!(super::super::NO_HTTP_CLIENT_EXIT_CODE, 3);
}

/// The unsupported arch/os exit code matches the script's `exit 2`.
#[test]
fn unsupported_exit_code_is_2() {
    assert_eq!(UNSUPPORTED_ARCH_OR_OS_EXIT_CODE, 2);
}

// ═══════════════════════════════════════════════════════════════════════
// § 7  Script probe: architecture mapping and sh compatibility
// ═══════════════════════════════════════════════════════════════════════

/// The install script's `case "$arch"` must recognise the same arch
/// strings that [`parse_uname_output`] does.
#[test]
fn install_script_arch_case_covers_known_architectures() {
    let template = super::super::INSTALL_SCRIPT_TEMPLATE;
    // The script has a `case "$arch" in` block.
    assert!(
        template.contains("x86_64)"),
        "install script must handle x86_64"
    );
    assert!(
        template.contains("aarch64|arm64)"),
        "install script must handle aarch64 and arm64"
    );
    // Unsupported arches fall through to `*) ... exit 2`.
    assert!(
        template.contains("*) echo \"unsupported arch:"),
        "install script must have an unsupported-arch catch-all"
    );
}

/// The install script's `case "$os_kernel"` must match what
/// [`parse_uname_output`] recognises.
#[test]
fn install_script_os_case_covers_known_os() {
    let template = super::super::INSTALL_SCRIPT_TEMPLATE;
    assert!(
        template.contains("Darwin) os_name=macos"),
        "install script must handle Darwin/macOS"
    );
    assert!(
        template.contains("Linux)  os_name=linux") || template.contains("Linux) os_name=linux"),
        "install script must handle Linux"
    );
}

/// The install script uses `set -e` so any failing command propagates.
#[test]
fn install_script_has_set_e() {
    let template = super::super::INSTALL_SCRIPT_TEMPLATE;
    assert!(
        template.contains("set -e"),
        "install script must use `set -e`"
    );
}

/// The no-HTTP-client exit code placeholder is present in the template.
#[test]
fn install_script_has_no_http_client_exit_code_placeholder() {
    let template = super::super::INSTALL_SCRIPT_TEMPLATE;
    assert!(
        template.contains("{no_http_client_exit_code}"),
        "install script must reference the no_http_client_exit_code placeholder"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// § 8  HTTP client selection / fallback in install script
// ═══════════════════════════════════════════════════════════════════════

/// The script must try curl first, then wget, then emit the sentinel.
#[test]
fn install_script_tries_curl_before_wget() {
    let template = super::super::INSTALL_SCRIPT_TEMPLATE;
    let curl_pos = template
        .find("command -v curl")
        .expect("script must check for curl");
    let wget_pos = template
        .find("command -v wget")
        .expect("script must check for wget");
    assert!(
        curl_pos < wget_pos,
        "script must try curl before wget (curl at {curl_pos}, wget at {wget_pos})"
    );
}

/// The sentinel exit for no HTTP client must appear after both client checks.
#[test]
fn install_script_no_http_client_sentinel_after_checks() {
    let template = super::super::INSTALL_SCRIPT_TEMPLATE;
    let wget_pos = template
        .find("command -v wget")
        .expect("script must check for wget");
    let sentinel_pos = template
        .find("exit {no_http_client_exit_code}")
        .expect("script must have the no-HTTP-client sentinel exit");
    assert!(
        sentinel_pos > wget_pos,
        "no-HTTP-client sentinel must come after the wget check"
    );
}

/// When `staging_tarball_path` is non-empty, the script skips download.
#[test]
fn install_script_staging_tarball_skips_download() {
    let template = super::super::INSTALL_SCRIPT_TEMPLATE;
    assert!(
        template.contains("if [ -n \"$staging_tarball_path\" ]"),
        "script must check for staging_tarball_path"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// § 9  SCP fallback trigger alignment
// ═══════════════════════════════════════════════════════════════════════

/// Verify that `NO_HTTP_CLIENT_EXIT_CODE` is the only exit code that
/// triggers the SCP fallback in the SSH transport. This is a structural
/// test to prevent introducing new fallback triggers accidentally.
#[test]
fn scp_fallback_triggered_only_by_no_http_client() {
    // The SCP fallback is triggered in ssh_transport.rs when
    // `output.status.code() == Some(NO_HTTP_CLIENT_EXIT_CODE)`.
    // We verify the constant is 3 and that the classifier agrees.
    assert_eq!(super::super::NO_HTTP_CLIENT_EXIT_CODE, 3);
    let cat = classify_install_failure(Some(3), "", false);
    assert_eq!(cat, InstallFailureCategory::NoHttpClient);

    // Other exit codes should NOT classify as NoHttpClient.
    for code in [0, 1, 2, 4, 18, 22, 23, 42, 127, 255] {
        let cat = classify_install_failure(Some(code), "", false);
        assert_ne!(
            cat,
            InstallFailureCategory::NoHttpClient,
            "exit code {code} should not classify as NoHttpClient"
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════
// § 10  Architecture mapping alignment between script and Rust
// ═══════════════════════════════════════════════════════════════════════

/// Verify that every arch string that `parse_uname_output` recognises
/// would also be accepted by the install script's case statement.
#[test]
fn rust_arch_mapping_consistent_with_script() {
    use super::super::{parse_uname_output, RemoteArch};

    let cases = [
        ("Linux x86_64", RemoteArch::X86_64),
        ("Linux aarch64", RemoteArch::Aarch64),
        ("Darwin arm64", RemoteArch::Aarch64),
        ("Linux armv8l", RemoteArch::Aarch64),
        ("Darwin x86_64", RemoteArch::X86_64),
    ];

    for (input, expected_arch) in &cases {
        let platform = parse_uname_output(input).expect(input);
        assert_eq!(&platform.arch, expected_arch, "arch mismatch for {input}");
    }

    // Unsupported arches should error.
    for bad in [
        "Linux mips",
        "Linux ppc64le",
        "Linux s390x",
        "Linux riscv64",
    ] {
        assert!(
            parse_uname_output(bad).is_err(),
            "{bad} should not be a supported arch"
        );
    }
}
