use super::*;
use warpui::r#async::BoxFuture;

fn static_auth_context() -> Arc<RemoteServerAuthContext> {
    Arc::new(RemoteServerAuthContext::new(
        || -> BoxFuture<'static, Option<String>> { Box::pin(async { None }) },
        || "user id/with spaces".to_string(),
        String::new(),
        String::new(),
        true,
    ))
}

#[test]
fn remote_proxy_command_quotes_identity_key() {
    let transport = SshTransport::new(
        PathBuf::from("/tmp/control-master.sock"),
        static_auth_context(),
    );

    let command = transport.remote_proxy_command();

    assert!(command.contains("remote-server-proxy --identity-key"));
    assert!(command.contains("'user id/with spaces'"));
}

#[test]
fn scp_fallback_skipped_for_unsupported_arch_os() {
    // Exit code 2 = unsupported arch/OS — SCP can't fix that.
    assert!(should_skip_scp_fallback(2));
}

#[test]
fn scp_fallback_not_skipped_for_no_http_client() {
    // Exit code 3 = no curl/wget — SCP fallback should trigger.
    assert!(!should_skip_scp_fallback(
        remote_server::setup::NO_HTTP_CLIENT_EXIT_CODE
    ));
}

#[test]
fn scp_fallback_not_skipped_for_curl_failures() {
    // curl exit code 6 = DNS resolution failure.
    assert!(!should_skip_scp_fallback(6));
    // curl exit code 7 = connection refused.
    assert!(!should_skip_scp_fallback(7));
    // curl exit code 28 = operation timed out.
    assert!(!should_skip_scp_fallback(28));
    // curl exit code 60 = SSL certificate problem.
    assert!(!should_skip_scp_fallback(60));
    // curl exit code 77 = CA cert error.
    assert!(!should_skip_scp_fallback(77));
}

#[test]
fn scp_fallback_not_skipped_for_generic_failures() {
    // Exit code 1 = generic script failure (e.g. "no binary found").
    assert!(!should_skip_scp_fallback(1));
    // Exit code 0 should not skip (though it won't reach this path
    // in practice since success is handled earlier).
    assert!(!should_skip_scp_fallback(0));
    // Negative exit code (signal-killed).
    assert!(!should_skip_scp_fallback(-1));
}
