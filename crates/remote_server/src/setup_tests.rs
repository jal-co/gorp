use super::*;

#[test]
fn parse_uname_linux_x86_64() {
    let platform = parse_uname_output("Linux x86_64").unwrap();
    assert_eq!(platform.os, RemoteOs::Linux);
    assert_eq!(platform.arch, RemoteArch::X86_64);
}

#[test]
fn parse_uname_linux_aarch64() {
    let platform = parse_uname_output("Linux aarch64").unwrap();
    assert_eq!(platform.os, RemoteOs::Linux);
    assert_eq!(platform.arch, RemoteArch::Aarch64);
}

#[test]
fn parse_uname_darwin_arm64() {
    let platform = parse_uname_output("Darwin arm64").unwrap();
    assert_eq!(platform.os, RemoteOs::MacOs);
    assert_eq!(platform.arch, RemoteArch::Aarch64);
}

#[test]
fn parse_uname_darwin_x86_64() {
    let platform = parse_uname_output("Darwin x86_64").unwrap();
    assert_eq!(platform.os, RemoteOs::MacOs);
    assert_eq!(platform.arch, RemoteArch::X86_64);
}

#[test]
fn parse_uname_linux_armv8l() {
    let platform = parse_uname_output("Linux armv8l").unwrap();
    assert_eq!(platform.os, RemoteOs::Linux);
    assert_eq!(platform.arch, RemoteArch::Aarch64);
}

#[test]
fn parse_uname_skips_shell_initialization_output() {
    let output = "Last login: Mon Apr  7 10:00:00 2025\nWelcome to Ubuntu\nLinux x86_64";
    let platform = parse_uname_output(output).unwrap();
    assert_eq!(platform.os, RemoteOs::Linux);
    assert_eq!(platform.arch, RemoteArch::X86_64);
}

#[test]
fn parse_uname_trims_whitespace() {
    let platform = parse_uname_output("  Linux x86_64  \n").unwrap();
    assert_eq!(platform.os, RemoteOs::Linux);
    assert_eq!(platform.arch, RemoteArch::X86_64);
}

#[test]
fn parse_uname_unsupported_os() {
    let result = parse_uname_output("Windows x86_64");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("unsupported OS"));
}

#[test]
fn parse_uname_unsupported_arch() {
    let result = parse_uname_output("Linux mips");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("unsupported arch"));
}

#[test]
fn parse_uname_empty_output() {
    let result = parse_uname_output("");
    assert!(result.is_err());
}

#[test]
fn parse_uname_missing_arch() {
    let result = parse_uname_output("Linux");
    assert!(result.is_err());
}

#[test]
fn state_is_ready() {
    assert!(RemoteServerSetupState::Ready.is_ready());
    assert!(!RemoteServerSetupState::Checking.is_ready());
    assert!(!RemoteServerSetupState::Initializing.is_ready());
}

#[test]
fn state_is_failed() {
    assert!(RemoteServerSetupState::Failed {
        error: "test".into()
    }
    .is_failed());
    assert!(!RemoteServerSetupState::Ready.is_failed());
}

#[test]
fn state_is_terminal() {
    assert!(RemoteServerSetupState::Ready.is_terminal());
    assert!(RemoteServerSetupState::Failed {
        error: "test".into()
    }
    .is_terminal());
    assert!(RemoteServerSetupState::Unsupported {
        reason: UnsupportedReason::NonGlibc {
            name: "musl".into()
        }
    }
    .is_terminal());
    assert!(!RemoteServerSetupState::Checking.is_terminal());
    assert!(!RemoteServerSetupState::Installing {
        progress_percent: None,
    }
    .is_terminal());
    assert!(!RemoteServerSetupState::Updating.is_terminal());
    assert!(!RemoteServerSetupState::Initializing.is_terminal());
}

#[test]
fn parse_preinstall_supported_glibc() {
    let stdout = "required_glibc=2.31\n\
                  libc_family=glibc\n\
                  libc_version=2.35\n\
                  status=supported\n";
    let result = PreinstallCheckResult::parse(stdout);
    assert_eq!(result.status, PreinstallStatus::Supported);
    assert_eq!(result.libc, RemoteLibc::Glibc(GlibcVersion::new(2, 35)));
    assert!(result.is_supported());
}

#[test]
fn parse_preinstall_unsupported_glibc_too_old() {
    let stdout = "required_glibc=2.31\n\
                  libc_family=glibc\n\
                  libc_version=2.17\n\
                  status=unsupported\n\
                  reason=glibc_too_old\n";
    let result = PreinstallCheckResult::parse(stdout);
    assert_eq!(
        result.status,
        PreinstallStatus::Unsupported {
            reason: UnsupportedReason::GlibcTooOld {
                detected: GlibcVersion::new(2, 17),
                required: GlibcVersion::new(2, 31),
            }
        }
    );
    assert!(!result.is_supported());
}

#[test]
fn parse_preinstall_unsupported_non_glibc() {
    let stdout = "required_glibc=2.31\n\
                  libc_family=musl\n\
                  status=unsupported\n\
                  reason=non_glibc\n";
    let result = PreinstallCheckResult::parse(stdout);
    assert_eq!(
        result.status,
        PreinstallStatus::Unsupported {
            reason: UnsupportedReason::NonGlibc {
                name: "musl".to_string()
            }
        }
    );
    assert_eq!(
        result.libc,
        RemoteLibc::NonGlibc {
            name: "musl".to_string()
        }
    );
    assert!(!result.is_supported());
}

/// Regression: the install script's tilde-expansion logic must work
/// across the bash versions we actually invoke at install time
/// (`run_ssh_script` pipes the script into `bash -s` on the remote).
/// Two interpreter quirks have to be avoided simultaneously:
///
///   1. bash 3.2 (macOS `/bin/bash`) keeps inner double-quotes around
///      the replacement of `${var/pattern/replacement}` literal, so
///      `"$HOME"` ends up as 6 literal characters and the install
///      lands under a directory tree literally named `"`.
///   2. bash 5.2+ with `patsub_replacement` (default-on) treats `&`
///      in the replacement as the matched pattern, so a `$HOME`
///      containing `&` resolves to a `~`-substituted path.
///
/// Both bugs surface as the install binary landing somewhere Warp's
/// launch step doesn't look, producing a misleading "Response channel
/// closed before receiving a reply".
///
/// This test drives the *actual* production script (via
/// [`install_script`]) rather than a hand-copied snippet, and runs it
/// against several `HOME` values to exercise the patsub-`&` trap as
/// well as the quote-literal trap. We truncate just before `mkdir -p`
/// so no filesystem side effects leak out of the test, and append a
/// marker `printf` to capture the resolved `install_dir`.
///
/// Gated to Unix because the test invokes `/bin/bash` (or `bash` from
/// PATH) directly. The bug only matters on Unix remotes anyway —
/// Warp's remote-server SSH wrapper doesn't target Windows hosts.
#[cfg(unix)]
#[test]
fn install_script_tilde_expansion_resolves_correctly() {
    use command::blocking::Command;
    use std::process::Stdio;

    let bash = if std::path::Path::new("/bin/bash").exists() {
        "/bin/bash"
    } else {
        "bash"
    };

    let script = install_script(None);
    let cutoff = script.find("mkdir -p \"$install_dir\"").expect(
        "install script no longer contains the `mkdir -p \"$install_dir\"` \
         checkpoint this test relies on; update the test alongside the \
         script change",
    );
    let probe = format!(
        "{prefix}\nprintf '%s' \"$install_dir\"\nexit 0\n",
        prefix = &script[..cutoff],
    );

    // Run the probe against a matrix of HOME values. The first is an
    // ordinary path; the second contains `&`, which exercises bash
    // 5.2's patsub_replacement (where it would otherwise expand to
    // the matched `~`).
    let cases = [
        ("/Users/test", "ordinary HOME"),
        (
            "/Users/A&B",
            "HOME with `&` (bash 5.2 patsub_replacement trap)",
        ),
    ];

    for (fake_home, label) in cases {
        let output = Command::new(bash)
            .arg("-c")
            .arg(&probe)
            .env("HOME", fake_home)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .expect("failed to spawn bash");

        assert!(
            output.status.success(),
            "[{label}] bash exited with {:?}: stderr={}",
            output.status,
            String::from_utf8_lossy(&output.stderr),
        );

        let install_dir = String::from_utf8_lossy(&output.stdout);
        assert!(
            !install_dir.contains('"'),
            "[{label}] install_dir contains literal quote characters \
             (bash 3.2 quote-literal regression): {install_dir:?}",
        );

        // Cross-check against the production layout: tilde must
        // resolve to HOME, so the result equals `remote_server_dir()`
        // with the leading `~` replaced.
        let expected = remote_server_dir().replacen('~', fake_home, 1);
        assert_eq!(
            install_dir, expected,
            "[{label}] install_dir resolved incorrectly",
        );
    }
}

/// Regression: guards against re-introducing the
/// `${var/pattern/replacement}` tilde-substitution form, which has two
/// known interpreter bugs (see
/// [`install_script_tilde_expansion_resolves_correctly`] for details).
/// Complements the live bash test — the live test catches behavioural
/// regressions, this static check fails fast and explains *why* in
/// the assertion message so a future contributor doesn't have to
/// re-discover the constraints from a CI failure.
#[test]
fn install_script_avoids_pattern_substitution_for_tilde_expansion() {
    let template = INSTALL_SCRIPT_TEMPLATE;
    assert!(
        !template.contains(r"/#\~/"),
        "install_remote_server.sh uses `${{var/#\\~/...}}` for tilde \
         expansion. This form has two known interpreter bugs that \
         silently mis-resolve the install path:\n\
         \n\
           1. bash 3.2 (macOS /bin/bash) keeps inner double-quotes \
              around the replacement literal, so `\"$HOME\"` ends up \
              as 6 literal characters including the quotes.\n\
           2. bash 5.2+ enables `patsub_replacement` by default, which \
              makes `&` in the replacement expand to the matched \
              pattern, so a `$HOME` containing `&` resolves wrong.\n\
         \n\
         Use `case`/`${{var#\\~}}` instead — see install_remote_server.sh \
         for the pattern.",
    );
}

#[test]
fn parse_preinstall_missing_status_falls_open() {
    // Garbled / partial script output — missing status field. Confirms
    // the fail-open invariant: anything we can't positively classify as
    // unsupported degrades to Unknown and is treated as supported, so a
    // flaky probe doesn't block the install.
    let stdout = "libc_family=glibc\nlibc_version=2.35\n";
    let result = PreinstallCheckResult::parse(stdout);
    assert_eq!(result.status, PreinstallStatus::Unknown);
    assert!(result.is_supported());
}

// ---------- download-failure sentinel tests ----------

/// The sentinel exit codes must not collide with each other or with
/// values that have reserved meaning in bash (0 = success, 1 = generic
/// failure, 2 = misuse / our "unsupported arch/OS" exit). This is a
/// compile-time-visible static check.
#[test]
fn sentinel_exit_codes_are_distinct() {
    assert_ne!(NO_HTTP_CLIENT_EXIT_CODE, DOWNLOAD_FAILED_EXIT_CODE);
    // Neither sentinel should shadow the script's own non-sentinel exits.
    for sentinel in [NO_HTTP_CLIENT_EXIT_CODE, DOWNLOAD_FAILED_EXIT_CODE] {
        assert_ne!(sentinel, 0, "sentinel must not be 0 (success)");
        assert_ne!(sentinel, 1, "sentinel must not be 1 (generic failure)");
        assert_ne!(sentinel, 2, "sentinel must not be 2 (unsupported arch/OS)");
    }
}

/// Static check: the install script template must contain the
/// `WARP_DOWNLOAD_FAILED` sentinel marker so the Rust side can
/// identify download failures in stderr diagnostics.
#[test]
fn install_script_contains_download_failed_sentinel_marker() {
    assert!(
        INSTALL_SCRIPT_TEMPLATE.contains("WARP_DOWNLOAD_FAILED"),
        "install_remote_server.sh must emit the WARP_DOWNLOAD_FAILED \
         marker to stderr when curl/wget fails so the client can \
         identify download-specific failures in logs",
    );
}

/// Static check: the download path must capture the tool's exit code
/// via `|| dl_exit=$?` rather than letting `set -e` abort with the
/// raw curl/wget exit code. Without this, the Rust side sees curl's
/// native exit code (e.g. 6 for DNS, 60 for TLS) instead of our
/// sentinel, and the SCP fallback never triggers.
#[test]
fn install_script_captures_download_exit_code() {
    assert!(
        INSTALL_SCRIPT_TEMPLATE.contains("|| dl_exit=$?"),
        "install_remote_server.sh must capture the download tool's exit \
         code via `|| dl_exit=$?` instead of letting `set -e` propagate \
         the raw exit code. Without this, download failures won't map \
         to the DOWNLOAD_FAILED sentinel and SCP fallback won't trigger.",
    );
}

/// Static check: curl must use `--connect-timeout` to bound the DNS +
/// TCP handshake phase. Without this, hosts with broken DNS can stall
/// the install indefinitely until the outer SSH timeout kills it,
/// which surfaces as a generic timeout rather than a download failure.
#[test]
fn install_script_curl_has_connect_timeout() {
    assert!(
        INSTALL_SCRIPT_TEMPLATE.contains("--connect-timeout"),
        "install_remote_server.sh must pass --connect-timeout to curl \
         so DNS/TCP-level failures surface quickly as recoverable \
         download errors rather than stalling until the SSH timeout.",
    );
}

/// The rendered install script must substitute both sentinel
/// placeholders with their numeric values.
#[test]
fn install_script_substitutes_download_failed_exit_code() {
    let script = install_script(None);
    // The rendered script must not contain the raw placeholder.
    assert!(
        !script.contains("{download_failed_exit_code}"),
        "install_script() must substitute {{download_failed_exit_code}} placeholder",
    );
    // It should contain the numeric literal for the sentinel.
    assert!(
        script.contains(&format!("exit {DOWNLOAD_FAILED_EXIT_CODE}")),
        "rendered install script must contain `exit {DOWNLOAD_FAILED_EXIT_CODE}`",
    );
}

/// Regression: the `dl_exit` check in the install script must compare
/// against 0 (not empty string) to correctly detect download failures.
/// A common shell bug is `[ "$var" -ne 0 ]` failing when `$var` is
/// unset — our script initializes `dl_exit=0` to avoid this.
#[test]
fn install_script_initializes_dl_exit() {
    assert!(
        INSTALL_SCRIPT_TEMPLATE.contains("dl_exit=0"),
        "install_remote_server.sh must initialize dl_exit=0 before the \
         download attempt so the subsequent `[ \"$dl_exit\" -ne 0 ]` \
         check doesn't fail on an unset variable.",
    );
}

/// Shell-level test: runs the download path of the production install
/// script against a guaranteed-unreachable URL and verifies that the
/// script exits with DOWNLOAD_FAILED_EXIT_CODE (not curl's native
/// exit code). This exercises the full sentinel flow end-to-end.
///
/// Gated to Unix (the script targets remote Unix hosts) and requires
/// `curl` on the test runner.
#[cfg(unix)]
#[test]
fn install_script_download_failure_exits_with_sentinel() {
    use command::blocking::Command;
    use std::process::Stdio;

    // Skip if curl isn't available on the test runner.
    let has_curl = Command::new("command")
        .args(["-v", "curl"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if !has_curl {
        eprintln!("skipping: curl not available on test runner");
        return;
    }

    let bash = if std::path::Path::new("/bin/bash").exists() {
        "/bin/bash"
    } else {
        "bash"
    };

    // Build a script that will definitely fail the download:
    // point at a non-routable IP so curl fails fast with a connect error.
    let script = install_script(None);

    // Replace the download URL with a guaranteed-failing one. The
    // rendered script has the real URL; swap it for 192.0.2.1 (TEST-NET,
    // RFC 5737 — guaranteed non-routable).
    let bad_script = script
        .replace(
            &download_url(),
            "http://192.0.2.1:1/download/cli",
        );

    // We need to truncate after the download sentinel check but before
    // tar (which would fail on the missing file). Insert an early exit
    // right before `tar`.
    let bad_script = bad_script.replace(
        "tar -xzf",
        "echo 'should not reach here' >&2; exit 99\ntar -xzf",
    );

    let output = Command::new(bash)
        .arg("-c")
        .arg(&bad_script)
        .env("HOME", "/tmp")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("failed to spawn bash");

    let exit_code = output.status.code().unwrap_or(-1);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(
        exit_code, DOWNLOAD_FAILED_EXIT_CODE,
        "install script should exit with DOWNLOAD_FAILED_EXIT_CODE ({DOWNLOAD_FAILED_EXIT_CODE}) \
         on curl failure, but exited with {exit_code}.\nstderr: {stderr}",
    );

    // The sentinel marker must appear in stderr for diagnostics.
    assert!(
        stderr.contains("WARP_DOWNLOAD_FAILED"),
        "stderr must contain WARP_DOWNLOAD_FAILED marker for diagnostics.\nstderr: {stderr}",
    );
}
