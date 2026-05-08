use super::*;

// ===== SSH disconnect / exit 255 =====

#[test]
fn ssh_disconnect_exit_255() {
    let cat = classify_install_failure("", Some(255));
    assert_eq!(cat, InstallFailureCategory::SshDisconnect);
}

#[test]
fn ssh_disconnect_connection_reset() {
    let cat = classify_install_failure(
        "ssh: Connection reset by peer",
        Some(1),
    );
    assert_eq!(cat, InstallFailureCategory::SshDisconnect);
}

#[test]
fn ssh_disconnect_broken_pipe() {
    let cat = classify_install_failure(
        "write failed: Broken pipe",
        Some(1),
    );
    assert_eq!(cat, InstallFailureCategory::SshDisconnect);
}

#[test]
fn ssh_disconnect_packet_write_wait() {
    let cat = classify_install_failure(
        "packet_write_wait: Connection to 10.0.0.1 port 22: Broken pipe",
        Some(255),
    );
    assert_eq!(cat, InstallFailureCategory::SshDisconnect);
}

#[test]
fn ssh_disconnect_client_loop() {
    let cat = classify_install_failure(
        "client_loop: send disconnect: Broken pipe",
        Some(1),
    );
    assert_eq!(cat, InstallFailureCategory::SshDisconnect);
}

#[test]
fn ssh_disconnect_connection_closed_by_remote() {
    let cat = classify_install_failure(
        "Connection closed by remote host",
        Some(1),
    );
    assert_eq!(cat, InstallFailureCategory::SshDisconnect);
}

// ===== Timeout =====

#[test]
fn timeout_operation_timed_out() {
    let cat = classify_install_failure(
        "curl: (28) Operation timed out after 60001 milliseconds",
        Some(28),
    );
    assert_eq!(cat, InstallFailureCategory::Timeout);
}

#[test]
fn timeout_connection_timed_out() {
    let cat = classify_install_failure(
        "curl: (28) Connection timed out after 15000 milliseconds",
        Some(28),
    );
    assert_eq!(cat, InstallFailureCategory::Timeout);
}

#[test]
fn timeout_generic() {
    let cat = classify_install_failure("command timeout reached", Some(124));
    assert_eq!(cat, InstallFailureCategory::Timeout);
}

// ===== Expired password =====

#[test]
fn expired_password_your_password_has_expired() {
    let cat = classify_install_failure(
        "WARNING: Your password has expired.\nPassword change required but no TTY available.",
        Some(1),
    );
    assert_eq!(cat, InstallFailureCategory::ExpiredPassword);
}

#[test]
fn expired_password_required_to_change() {
    let cat = classify_install_failure(
        "You are required to change your password immediately (administrator enforced)",
        Some(1),
    );
    assert_eq!(cat, InstallFailureCategory::ExpiredPassword);
}

// ===== Missing HTTP clients =====

#[test]
fn missing_http_client_neither() {
    let cat = classify_install_failure(
        "error: neither curl nor wget is available",
        Some(3),
    );
    assert_eq!(cat, InstallFailureCategory::MissingHttpClient);
}

#[test]
fn missing_http_client_exit_code_only() {
    let cat = classify_install_failure("", Some(3));
    assert_eq!(cat, InstallFailureCategory::MissingHttpClient);
}

#[test]
fn missing_curl_command_not_found() {
    let cat = classify_install_failure(
        "bash: curl: command not found",
        Some(127),
    );
    assert_eq!(cat, InstallFailureCategory::MissingCurl);
}

#[test]
fn missing_wget_command_not_found() {
    let cat = classify_install_failure(
        "bash: wget: command not found",
        Some(127),
    );
    assert_eq!(cat, InstallFailureCategory::MissingWget);
}

// ===== Missing tar =====

#[test]
fn missing_tar_not_found() {
    let cat = classify_install_failure(
        "tar: command not found",
        Some(127),
    );
    assert_eq!(cat, InstallFailureCategory::MissingTar);
}

#[test]
fn missing_tar_colon_not_found() {
    let cat = classify_install_failure(
        "bash: tar: not found",
        Some(127),
    );
    assert_eq!(cat, InstallFailureCategory::MissingTar);
}

// ===== Missing bash =====

#[test]
fn missing_bash_not_found() {
    let cat = classify_install_failure(
        "bash: command not found",
        Some(127),
    );
    assert_eq!(cat, InstallFailureCategory::MissingBash);
}

#[test]
fn missing_bash_no_such_file() {
    let cat = classify_install_failure(
        "bash: No such file or directory",
        Some(127),
    );
    assert_eq!(cat, InstallFailureCategory::MissingBash);
}

// ===== Unsupported arch/OS =====

#[test]
fn unsupported_architecture() {
    let cat = classify_install_failure(
        "unsupported arch: mips",
        Some(2),
    );
    assert_eq!(cat, InstallFailureCategory::UnsupportedArchitecture);
}

#[test]
fn unsupported_os() {
    let cat = classify_install_failure(
        "unsupported OS: FreeBSD",
        Some(2),
    );
    assert_eq!(cat, InstallFailureCategory::UnsupportedOs);
}

// ===== DNS failure =====

#[test]
fn dns_could_not_resolve_host() {
    let cat = classify_install_failure(
        "curl: (6) Could not resolve host: app.warp.dev",
        Some(6),
    );
    assert_eq!(cat, InstallFailureCategory::DnsFailure);
}

#[test]
fn dns_name_or_service_not_known() {
    let cat = classify_install_failure(
        "wget: unable to resolve host address 'app.warp.dev'\nName or service not known",
        Some(4),
    );
    assert_eq!(cat, InstallFailureCategory::DnsFailure);
}

#[test]
fn dns_temporary_failure() {
    let cat = classify_install_failure(
        "Temporary failure in name resolution",
        Some(6),
    );
    assert_eq!(cat, InstallFailureCategory::DnsFailure);
}

// ===== Connection refused =====

#[test]
fn connection_refused() {
    let cat = classify_install_failure(
        "curl: (7) Failed to connect to app.warp.dev port 443: Connection refused",
        Some(7),
    );
    assert_eq!(cat, InstallFailureCategory::ConnectionRefused);
}

// ===== Connection unreachable =====

#[test]
fn connection_unreachable_no_route() {
    let cat = classify_install_failure(
        "curl: (7) Failed to connect to app.warp.dev port 443: No route to host",
        Some(7),
    );
    assert_eq!(cat, InstallFailureCategory::ConnectionUnreachable);
}

#[test]
fn connection_unreachable_network() {
    let cat = classify_install_failure(
        "curl: (7) Network is unreachable",
        Some(7),
    );
    assert_eq!(cat, InstallFailureCategory::ConnectionUnreachable);
}

// ===== TLS/CA failures =====

#[test]
fn tls_ssl_certificate_problem() {
    let cat = classify_install_failure(
        "curl: (60) SSL certificate problem: unable to get local issuer certificate",
        Some(60),
    );
    assert_eq!(cat, InstallFailureCategory::TlsCaFailure);
}

#[test]
fn tls_certificate_verify_failed() {
    let cat = classify_install_failure(
        "OpenSSL: error:14090086:SSL routines:ssl3_get_server_certificate:certificate verify failed",
        Some(1),
    );
    assert_eq!(cat, InstallFailureCategory::TlsCaFailure);
}

#[test]
fn tls_self_signed() {
    let cat = classify_install_failure(
        "curl: (60) SSL certificate problem: self signed certificate in chain",
        Some(60),
    );
    assert_eq!(cat, InstallFailureCategory::TlsCaFailure);
}

// ===== HTTP errors =====

#[test]
fn http_403_forbidden() {
    let cat = classify_install_failure(
        "curl: (22) The requested URL returned error: 403 Forbidden",
        Some(22),
    );
    assert_eq!(cat, InstallFailureCategory::HttpForbidden);
}

#[test]
fn http_502_bad_gateway() {
    let cat = classify_install_failure(
        "curl: (22) The requested URL returned error: 502 Bad Gateway",
        Some(22),
    );
    assert_eq!(cat, InstallFailureCategory::HttpBadGateway);
}

#[test]
fn http_generic_error() {
    let cat = classify_install_failure(
        "curl: (22) The requested URL returned error: 500 Internal Server Error",
        Some(22),
    );
    assert_eq!(cat, InstallFailureCategory::HttpError);
}

#[test]
fn http_wget_error() {
    let cat = classify_install_failure(
        "wget: ERROR 4 (NETWORK_FAILURE)",
        Some(4),
    );
    assert_eq!(cat, InstallFailureCategory::HttpError);
}

// ===== Partial download =====

#[test]
fn partial_download_curl_18() {
    let cat = classify_install_failure(
        "curl: (18) transfer closed with 12345 bytes remaining",
        Some(18),
    );
    assert_eq!(cat, InstallFailureCategory::PartialDownload);
}

#[test]
fn partial_download_partial_file() {
    let cat = classify_install_failure(
        "curl: (18) Partial file. Only 1024 of 5242880 bytes were received.",
        Some(18),
    );
    assert_eq!(cat, InstallFailureCategory::PartialDownload);
}

#[test]
fn partial_download_transfer_closed() {
    let cat = classify_install_failure(
        "transfer closed with outstanding read data remaining",
        Some(18),
    );
    assert_eq!(cat, InstallFailureCategory::PartialDownload);
}

// ===== Read-only filesystem =====

#[test]
fn read_only_filesystem() {
    let cat = classify_install_failure(
        "mkdir: cannot create directory '/home/user/.warp': Read-only file system",
        Some(1),
    );
    assert_eq!(cat, InstallFailureCategory::ReadOnlyFilesystem);
}

#[test]
fn read_only_filesystem_erofs() {
    let cat = classify_install_failure(
        "open failed: EROFS (Read-only file system)",
        Some(1),
    );
    assert_eq!(cat, InstallFailureCategory::ReadOnlyFilesystem);
}

// ===== No space left =====

#[test]
fn no_space_left() {
    let cat = classify_install_failure(
        "write error: No space left on device",
        Some(1),
    );
    assert_eq!(cat, InstallFailureCategory::NoSpaceLeft);
}

#[test]
fn disk_quota_exceeded() {
    let cat = classify_install_failure(
        "write: Disk quota exceeded",
        Some(1),
    );
    assert_eq!(cat, InstallFailureCategory::NoSpaceLeft);
}

// ===== Download write failure =====

#[test]
fn download_write_failure_curl_23() {
    let cat = classify_install_failure(
        "curl: (23) Failed writing body (0 != 16384)",
        Some(23),
    );
    assert_eq!(cat, InstallFailureCategory::DownloadWriteFailure);
}

#[test]
fn download_write_failure_failed_to_create() {
    let cat = classify_install_failure(
        "curl: failed to create file '/tmp/install.XXXX/oz.tar.gz'",
        Some(23),
    );
    assert_eq!(cat, InstallFailureCategory::DownloadWriteFailure);
}

// ===== Tar permission failure =====

#[test]
fn tar_permission_cannot_change_ownership() {
    let cat = classify_install_failure(
        "tar: oz: Cannot change ownership to uid 1000, gid 1000: Operation not permitted\ntar: Exiting with failure status due to previous errors",
        Some(2),
    );
    assert_eq!(cat, InstallFailureCategory::TarPermissionFailure);
}

#[test]
fn tar_permission_cannot_open() {
    let cat = classify_install_failure(
        "tar: ./oz: Cannot open: Permission denied",
        Some(2),
    );
    assert_eq!(cat, InstallFailureCategory::TarPermissionFailure);
}

// ===== Tar extraction failure =====

#[test]
fn tar_extraction_not_gzip() {
    let cat = classify_install_failure(
        "tar: (stdin): not in gzip format\ntar: Error is not recoverable: exiting now",
        Some(2),
    );
    assert_eq!(cat, InstallFailureCategory::TarExtractionFailure);
}

#[test]
fn tar_extraction_unexpected_eof() {
    let cat = classify_install_failure(
        "tar: Unexpected EOF in archive\ntar: Error is not recoverable: exiting now",
        Some(2),
    );
    assert_eq!(cat, InstallFailureCategory::TarExtractionFailure);
}

#[test]
fn tar_extraction_corrupted() {
    let cat = classify_install_failure(
        "tar: Archive is corrupted",
        Some(2),
    );
    assert_eq!(cat, InstallFailureCategory::TarExtractionFailure);
}

// ===== Install dir permission denied =====

#[test]
fn install_dir_permission_denied_mkdir() {
    let cat = classify_install_failure(
        "mkdir: cannot create directory '/home/user/.warp': Permission denied",
        Some(1),
    );
    assert_eq!(cat, InstallFailureCategory::InstallDirPermissionDenied);
}

#[test]
fn install_dir_permission_denied_mv() {
    let cat = classify_install_failure(
        "mv: cannot move '/tmp/.install.XXXX/oz' to '/home/user/.warp/remote-server/oz': Permission denied",
        Some(1),
    );
    assert_eq!(cat, InstallFailureCategory::InstallDirPermissionDenied);
}

#[test]
fn install_dir_permission_denied_chmod() {
    let cat = classify_install_failure(
        "chmod: changing permissions of '/home/user/.warp/remote-server/oz': Permission denied",
        Some(1),
    );
    assert_eq!(cat, InstallFailureCategory::InstallDirPermissionDenied);
}

#[test]
fn install_dir_generic_permission_denied() {
    let cat = classify_install_failure(
        "/home/user/.warp/remote-server: Permission denied",
        Some(1),
    );
    assert_eq!(cat, InstallFailureCategory::InstallDirPermissionDenied);
}

// ===== Startup file permission denied =====

#[test]
fn startup_file_bashrc_permission_denied() {
    let cat = classify_install_failure(
        "/home/user/.bashrc: Permission denied",
        Some(1),
    );
    assert_eq!(cat, InstallFailureCategory::StartupFilePermissionDenied);
}

#[test]
fn startup_file_profile_permission_denied() {
    let cat = classify_install_failure(
        "bash: /home/user/.profile: Permission denied",
        Some(1),
    );
    assert_eq!(cat, InstallFailureCategory::StartupFilePermissionDenied);
}

// ===== Script error (generic) =====

#[test]
fn script_error_unknown_exit_code() {
    let cat = classify_install_failure(
        "some unknown error happened",
        Some(42),
    );
    assert_eq!(cat, InstallFailureCategory::ScriptError);
}

// ===== Unknown =====

#[test]
fn unknown_no_exit_code_no_pattern() {
    let cat = classify_install_failure("", None);
    assert_eq!(cat, InstallFailureCategory::Unknown);
}

#[test]
fn unknown_exit_zero_no_pattern() {
    let cat = classify_install_failure("all good but still reported as failure", Some(0));
    assert_eq!(cat, InstallFailureCategory::Unknown);
}

// ===== as_str / title / description completeness =====

#[test]
fn all_variants_have_non_empty_title_and_description() {
    let variants = [
        InstallFailureCategory::Timeout,
        InstallFailureCategory::MissingCurl,
        InstallFailureCategory::MissingWget,
        InstallFailureCategory::MissingHttpClient,
        InstallFailureCategory::MissingTar,
        InstallFailureCategory::MissingBash,
        InstallFailureCategory::UnsupportedArchitecture,
        InstallFailureCategory::UnsupportedOs,
        InstallFailureCategory::DnsFailure,
        InstallFailureCategory::ConnectionRefused,
        InstallFailureCategory::ConnectionUnreachable,
        InstallFailureCategory::TlsCaFailure,
        InstallFailureCategory::HttpForbidden,
        InstallFailureCategory::HttpBadGateway,
        InstallFailureCategory::HttpError,
        InstallFailureCategory::PartialDownload,
        InstallFailureCategory::DownloadWriteFailure,
        InstallFailureCategory::InstallDirPermissionDenied,
        InstallFailureCategory::NoSpaceLeft,
        InstallFailureCategory::ReadOnlyFilesystem,
        InstallFailureCategory::TarExtractionFailure,
        InstallFailureCategory::TarPermissionFailure,
        InstallFailureCategory::ExpiredPassword,
        InstallFailureCategory::StartupFilePermissionDenied,
        InstallFailureCategory::SshDisconnect,
        InstallFailureCategory::ScriptError,
        InstallFailureCategory::Unknown,
    ];
    for v in &variants {
        assert!(!v.title().is_empty(), "empty title for {v:?}");
        assert!(!v.description().is_empty(), "empty description for {v:?}");
        assert!(!v.as_str().is_empty(), "empty as_str for {v:?}");
        // as_str should be snake_case (lowercase, underscores only)
        assert!(
            v.as_str().chars().all(|c| c.is_ascii_lowercase() || c == '_'),
            "as_str '{}' is not snake_case for {v:?}",
            v.as_str()
        );
    }
}

// ===== Display trait =====

#[test]
fn display_matches_as_str() {
    let cat = InstallFailureCategory::TlsCaFailure;
    assert_eq!(format!("{cat}"), cat.as_str());
}

// ===== ClassifiedInstallFailure =====

#[test]
fn classified_from_raw() {
    let f = ClassifiedInstallFailure::from_raw(
        "curl: (28) Operation timed out",
        Some(28),
    );
    assert_eq!(f.category, InstallFailureCategory::Timeout);
    assert_eq!(f.raw_stderr, "curl: (28) Operation timed out");
    assert_eq!(f.exit_code, Some(28));
}

#[test]
fn classified_from_error_string() {
    let f = ClassifiedInstallFailure::from_error_string(
        "install script failed (exit 6): curl: (6) Could not resolve host: app.warp.dev",
    );
    assert_eq!(f.category, InstallFailureCategory::DnsFailure);
    assert_eq!(f.exit_code, Some(6));
    assert!(f.raw_stderr.contains("Could not resolve host"));
}

#[test]
fn classified_from_error_string_no_exit_pattern() {
    let f = ClassifiedInstallFailure::from_error_string(
        "some random error without exit code pattern",
    );
    assert_eq!(f.category, InstallFailureCategory::Unknown);
    assert_eq!(f.exit_code, None);
}

#[test]
fn classified_from_error_string_ssh_exit_255() {
    let f = ClassifiedInstallFailure::from_error_string(
        "install script failed (exit 255): Connection closed by remote host",
    );
    assert_eq!(f.category, InstallFailureCategory::SshDisconnect);
    assert_eq!(f.exit_code, Some(255));
}

// ===== parse_error_string =====

#[test]
fn parse_error_string_standard_format() {
    let (code, stderr) = parse_error_string(
        "install script failed (exit 1): mkdir: Permission denied",
    );
    assert_eq!(code, Some(1));
    assert_eq!(stderr, "mkdir: Permission denied");
}

#[test]
fn parse_error_string_no_pattern() {
    let (code, stderr) = parse_error_string("just some error text");
    assert_eq!(code, None);
    assert_eq!(stderr, "just some error text");
}

#[test]
fn parse_error_string_negative_exit_code() {
    let (code, stderr) = parse_error_string(
        "install script failed (exit -1): terminated by signal",
    );
    assert_eq!(code, Some(-1));
    assert_eq!(stderr, "terminated by signal");
}

// ===== Edge cases =====

#[test]
fn case_insensitive_matching() {
    let cat = classify_install_failure(
        "CURL: (60) SSL CERTIFICATE PROBLEM: UNABLE TO GET LOCAL ISSUER CERTIFICATE",
        Some(60),
    );
    assert_eq!(cat, InstallFailureCategory::TlsCaFailure);
}

#[test]
fn mixed_case_unsupported_arch() {
    let cat = classify_install_failure(
        "Unsupported Arch: ppc64le",
        Some(2),
    );
    assert_eq!(cat, InstallFailureCategory::UnsupportedArchitecture);
}

#[test]
fn multiline_stderr_with_tar_and_permission() {
    // When tar stderr contains both ownership and exiting errors,
    // the more specific TarPermissionFailure should match.
    let cat = classify_install_failure(
        "tar: oz-dev: Cannot change ownership to uid 0, gid 0: Operation not permitted\n\
         tar: Exiting with failure status due to previous errors",
        Some(2),
    );
    assert_eq!(cat, InstallFailureCategory::TarPermissionFailure);
}
