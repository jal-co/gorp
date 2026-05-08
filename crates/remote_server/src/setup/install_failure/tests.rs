use super::*;

// -----------------------------------------------------------------------
// Unsupported platform
// -----------------------------------------------------------------------

#[test]
fn unsupported_arch_mips() {
    let kind = classify_install_error("unsupported arch: mips\n", Some(2));
    assert_eq!(
        kind,
        InstallFailureKind::UnsupportedPlatform {
            detail: "unsupported arch: mips".to_string()
        }
    );
    assert!(kind.is_unsupported_platform());
    assert!(!kind.is_retriable());
}

#[test]
fn unsupported_arch_armv7l() {
    let kind = classify_install_error("unsupported arch: armv7l\n", Some(2));
    assert!(kind.is_unsupported_platform());
}

#[test]
fn unsupported_os_freebsd() {
    let kind = classify_install_error("unsupported OS: FreeBSD\n", Some(2));
    assert_eq!(
        kind,
        InstallFailureKind::UnsupportedPlatform {
            detail: "unsupported OS: FreeBSD".to_string()
        }
    );
}

#[test]
fn unsupported_os_sunos() {
    let kind = classify_install_error("unsupported OS: SunOS\n", Some(2));
    assert!(kind.is_unsupported_platform());
}

#[test]
fn unsupported_exit_code_2_with_uname_hint() {
    // Some older install script variants emit uname-related text.
    let kind = classify_install_error("uname: unknown hardware platform\n", Some(2));
    assert!(kind.is_unsupported_platform());
}

// -----------------------------------------------------------------------
// Permission denied
// -----------------------------------------------------------------------

#[test]
fn permission_denied_mkdir() {
    let kind = classify_install_error(
        "mkdir: cannot create directory '/home/user/.warp/remote-server': Permission denied\n",
        Some(1),
    );
    assert_eq!(kind, InstallFailureKind::PermissionDenied);
    assert!(!kind.is_retriable());
}

#[test]
fn permission_denied_mv() {
    let kind = classify_install_error(
        "mv: cannot move '/tmp/install.xxx/oz' to '/opt/warp/oz': Permission denied\n",
        Some(1),
    );
    assert_eq!(kind, InstallFailureKind::PermissionDenied);
}

#[test]
fn permission_denied_eacces() {
    let kind = classify_install_error("tar: oz: EACCES\n", Some(1));
    assert_eq!(kind, InstallFailureKind::PermissionDenied);
}

#[test]
fn permission_denied_operation_not_permitted() {
    let kind = classify_install_error(
        "chmod: changing permissions of '/opt/oz': Operation not permitted\n",
        Some(1),
    );
    assert_eq!(kind, InstallFailureKind::PermissionDenied);
}

#[test]
fn permission_denied_cannot_create_directory() {
    let kind = classify_install_error(
        "cannot create directory '/root/.warp': permission denied\n",
        Some(1),
    );
    assert_eq!(kind, InstallFailureKind::PermissionDenied);
}

#[test]
fn permission_denied_does_not_match_ssh_publickey() {
    // SSH auth failures mention "Permission denied (publickey)" — these
    // are transport errors, not filesystem permission errors.
    let kind = classify_install_error(
        "Permission denied (publickey,keyboard-interactive).\n",
        Some(255),
    );
    // Should be SSH transport, not permission denied.
    assert!(matches!(kind, InstallFailureKind::SshTransportError { .. }));
}

// -----------------------------------------------------------------------
// Read-only filesystem
// -----------------------------------------------------------------------

#[test]
fn read_only_filesystem_standard() {
    let kind = classify_install_error(
        "mkdir: cannot create directory '/home/user/.warp': Read-only file system\n",
        Some(1),
    );
    assert_eq!(kind, InstallFailureKind::ReadOnlyFilesystem);
    assert!(!kind.is_retriable());
}

#[test]
fn read_only_filesystem_erofs() {
    let kind = classify_install_error("tar: oz: EROFS\n", Some(1));
    assert_eq!(kind, InstallFailureKind::ReadOnlyFilesystem);
}

#[test]
fn read_only_filesystem_no_hyphen() {
    // Some systems emit "Read only file system" without the hyphen.
    let kind = classify_install_error(
        "cp: cannot create regular file: Read only file system\n",
        Some(1),
    );
    assert_eq!(kind, InstallFailureKind::ReadOnlyFilesystem);
}

#[test]
fn read_only_takes_precedence_over_permission_denied() {
    // Some systems emit both — read-only should win.
    let kind = classify_install_error(
        "mkdir: cannot create directory '/opt/foo': Read-only file system\n\
         chmod: Permission denied\n",
        Some(1),
    );
    assert_eq!(kind, InstallFailureKind::ReadOnlyFilesystem);
}

// -----------------------------------------------------------------------
// Disk full / quota
// -----------------------------------------------------------------------

#[test]
fn disk_full_no_space_left() {
    let kind = classify_install_error("tar: oz: write error: No space left on device\n", Some(1));
    assert_eq!(kind, InstallFailureKind::DiskFull);
    assert!(!kind.is_retriable());
}

#[test]
fn disk_full_quota_exceeded() {
    let kind = classify_install_error("write: Disk quota exceeded\n", Some(1));
    assert_eq!(kind, InstallFailureKind::DiskFull);
}

#[test]
fn disk_full_enospc() {
    let kind = classify_install_error("mv: ENOSPC\n", Some(1));
    assert_eq!(kind, InstallFailureKind::DiskFull);
}

#[test]
fn disk_full_edquot() {
    let kind = classify_install_error("write: EDQUOT\n", Some(1));
    assert_eq!(kind, InstallFailureKind::DiskFull);
}

#[test]
fn disk_full_not_enough_space() {
    let kind = classify_install_error("cp: not enough space on device\n", Some(1));
    assert_eq!(kind, InstallFailureKind::DiskFull);
}

// -----------------------------------------------------------------------
// Account issues
// -----------------------------------------------------------------------

#[test]
fn account_password_expired() {
    let kind = classify_install_error(
        "WARNING: Your password has expired.\nYou must change your password now and login again!\n",
        Some(1),
    );
    assert_eq!(
        kind,
        InstallFailureKind::AccountIssue {
            detail: "password expired".to_string()
        }
    );
    assert!(!kind.is_retriable());
}

#[test]
fn account_password_expired_short() {
    let kind = classify_install_error("Password expired\n", Some(1));
    assert_eq!(
        kind,
        InstallFailureKind::AccountIssue {
            detail: "password expired".to_string()
        }
    );
}

#[test]
fn account_required_to_change_password() {
    let kind = classify_install_error(
        "You are required to change your password immediately (root enforced)\n",
        Some(1),
    );
    assert_eq!(
        kind,
        InstallFailureKind::AccountIssue {
            detail: "password expired".to_string()
        }
    );
}

#[test]
fn account_pam_chauthtok() {
    let kind = classify_install_error(
        "pam_chauthtok: Authentication token manipulation error\n",
        Some(1),
    );
    assert_eq!(
        kind,
        InstallFailureKind::AccountIssue {
            detail: "password expired".to_string()
        }
    );
}

#[test]
fn account_no_tty_present() {
    let kind = classify_install_error(
        "sudo: no tty present and no askpass program specified\n",
        Some(1),
    );
    assert_eq!(
        kind,
        InstallFailureKind::AccountIssue {
            detail: "no TTY available".to_string()
        }
    );
}

#[test]
fn account_stdin_not_terminal() {
    let kind = classify_install_error("stdin is not a terminal\n", Some(1));
    assert_eq!(
        kind,
        InstallFailureKind::AccountIssue {
            detail: "no TTY available".to_string()
        }
    );
}

#[test]
fn account_must_be_run_from_terminal() {
    let kind = classify_install_error("passwd: must be run from a terminal\n", Some(1));
    assert_eq!(
        kind,
        InstallFailureKind::AccountIssue {
            detail: "no TTY available".to_string()
        }
    );
}

// -----------------------------------------------------------------------
// SSH transport errors
// -----------------------------------------------------------------------

#[test]
fn ssh_exit_255_connection_reset() {
    let kind = classify_install_error("Connection reset by peer\n", Some(255));
    assert!(matches!(kind, InstallFailureKind::SshTransportError { .. }));
    assert!(kind.is_retriable());
}

#[test]
fn ssh_exit_255_empty_stderr() {
    // SSH sometimes exits 255 with no stderr at all.
    let kind = classify_install_error("", Some(255));
    assert!(matches!(kind, InstallFailureKind::SshTransportError { .. }));
}

#[test]
fn ssh_broken_pipe() {
    let kind = classify_install_error("Write failed: Broken pipe\n", Some(1));
    assert!(matches!(kind, InstallFailureKind::SshTransportError { .. }));
}

#[test]
fn ssh_connection_closed_by_remote() {
    let kind = classify_install_error("Connection closed by remote host\n", Some(255));
    assert!(matches!(kind, InstallFailureKind::SshTransportError { .. }));
}

#[test]
fn ssh_kex_exchange() {
    let kind = classify_install_error(
        "kex_exchange_identification: read: Connection reset by peer\n",
        Some(255),
    );
    assert!(matches!(kind, InstallFailureKind::SshTransportError { .. }));
}

#[test]
fn ssh_exchange_identification() {
    let kind = classify_install_error(
        "ssh_exchange_identification: Connection closed by remote host\n",
        Some(255),
    );
    assert!(matches!(kind, InstallFailureKind::SshTransportError { .. }));
}

#[test]
fn ssh_packet_write_wait() {
    let kind = classify_install_error(
        "packet_write_wait: Connection to 10.0.0.1 port 22: Broken pipe\n",
        Some(255),
    );
    assert!(matches!(kind, InstallFailureKind::SshTransportError { .. }));
}

#[test]
fn ssh_signal_kill_with_broken_pipe() {
    // Process killed by signal (exit_code = None), stderr has connection info.
    let kind = classify_install_error("Broken pipe\n", None);
    assert!(matches!(kind, InstallFailureKind::SshTransportError { .. }));
}

#[test]
fn ssh_signal_kill_with_connection_reset() {
    let kind = classify_install_error("Connection reset by 10.0.0.1 port 22\n", None);
    assert!(matches!(kind, InstallFailureKind::SshTransportError { .. }));
}

#[test]
fn ssh_signal_kill_with_connection_refused() {
    let kind = classify_install_error(
        "ssh: connect to host 10.0.0.1 port 22: Connection refused\n",
        None,
    );
    assert!(matches!(kind, InstallFailureKind::SshTransportError { .. }));
}

// -----------------------------------------------------------------------
// Timeout
// -----------------------------------------------------------------------

#[test]
fn timeout_timed_out() {
    let kind = classify_install_error("timed out after 60s\n", Some(124));
    assert_eq!(kind, InstallFailureKind::Timeout);
    assert!(kind.is_retriable());
}

#[test]
fn timeout_operation_timed_out() {
    let kind = classify_install_error(
        "curl: (28) Operation timed out after 30000 milliseconds\n",
        Some(28),
    );
    assert_eq!(kind, InstallFailureKind::Timeout);
}

#[test]
fn timeout_connection_timed_out_without_ssh_code() {
    // "Connection timed out" with a non-255 exit code → Timeout (not SSH transport).
    let kind = classify_install_error(
        "ssh: connect to host example.com port 22: Connection timed out\n",
        Some(1),
    );
    assert_eq!(kind, InstallFailureKind::Timeout);
}

#[test]
fn timeout_connection_timed_out_with_ssh_255() {
    // Exit code 255 takes priority over timeout classification.
    let kind = classify_install_error(
        "ssh: connect to host example.com port 22: Connection timed out\n",
        Some(255),
    );
    assert!(matches!(kind, InstallFailureKind::SshTransportError { .. }));
}

// -----------------------------------------------------------------------
// Unclassified (fallback)
// -----------------------------------------------------------------------

#[test]
fn unclassified_unknown_error() {
    let kind = classify_install_error("something completely unexpected happened\n", Some(42));
    assert!(matches!(kind, InstallFailureKind::Unclassified { .. }));
    assert!(kind.is_retriable());
}

#[test]
fn unclassified_empty_stderr() {
    let kind = classify_install_error("", Some(1));
    assert!(matches!(kind, InstallFailureKind::Unclassified { .. }));
}

#[test]
fn unclassified_truncates_long_message() {
    let long = "x".repeat(500);
    let kind = classify_install_error(&long, Some(1));
    match kind {
        InstallFailureKind::Unclassified { message } => {
            assert!(message.len() <= 260); // 256 + "…"
        }
        other => panic!("expected Unclassified, got {other:?}"),
    }
}

// -----------------------------------------------------------------------
// Display and serialization
// -----------------------------------------------------------------------

#[test]
fn display_permission_denied() {
    assert_eq!(
        InstallFailureKind::PermissionDenied.to_string(),
        "permission denied on install directory"
    );
}

#[test]
fn display_timeout() {
    assert_eq!(InstallFailureKind::Timeout.to_string(), "install timed out");
}

#[test]
fn display_unsupported_platform() {
    let kind = InstallFailureKind::UnsupportedPlatform {
        detail: "unsupported arch: mips".to_string(),
    };
    assert_eq!(
        kind.to_string(),
        "unsupported platform: unsupported arch: mips"
    );
}

// -----------------------------------------------------------------------
// Priority / precedence
// -----------------------------------------------------------------------

#[test]
fn ssh_255_takes_priority_over_permission_denied() {
    // If SSH itself failed (255), the permission denied in stderr is about
    // SSH auth, not filesystem permissions.
    let kind = classify_install_error("Permission denied\n", Some(255));
    assert!(matches!(kind, InstallFailureKind::SshTransportError { .. }));
}

#[test]
fn read_only_takes_priority_over_permission_denied_in_same_stderr() {
    let kind = classify_install_error(
        "mkdir: Permission denied\ntar: Read-only file system\n",
        Some(1),
    );
    assert_eq!(kind, InstallFailureKind::ReadOnlyFilesystem);
}

// -----------------------------------------------------------------------
// Helper unit tests
// -----------------------------------------------------------------------

#[test]
fn truncate_short_string_unchanged() {
    assert_eq!(truncate_for_display("hello", 10), "hello");
}

#[test]
fn truncate_long_string_adds_ellipsis() {
    let result = truncate_for_display("hello world", 5);
    assert_eq!(result, "hello…");
}

#[test]
fn contains_any_matches() {
    assert!(contains_any("foo bar baz", &["bar"]));
    assert!(!contains_any("foo bar baz", &["qux"]));
}

// -----------------------------------------------------------------------
// Multi-line / noisy stderr
// -----------------------------------------------------------------------

#[test]
fn permission_denied_in_noisy_output() {
    let stderr = "\
warning: something unrelated\n\
debug: connecting to host\n\
mkdir: cannot create directory '/home/.warp': Permission denied\n\
cleanup: done\n";
    let kind = classify_install_error(stderr, Some(1));
    assert_eq!(kind, InstallFailureKind::PermissionDenied);
}

#[test]
fn disk_full_among_other_errors() {
    let stderr = "\
  % Total    % Received % Xferd  Average Speed\n\
curl: (23) Failed writing body (0 != 16384)\n\
tar: oz: No space left on device\n";
    let kind = classify_install_error(stderr, Some(1));
    assert_eq!(kind, InstallFailureKind::DiskFull);
}
