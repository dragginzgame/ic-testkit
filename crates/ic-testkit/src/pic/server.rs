//! Release-bound PocketIC server policy, independent of client download defaults.

/// The server selected by this Testkit release's explicit provisioning CLI.
pub const POCKET_IC_SERVER_VERSION: &str = "16.1.0";

/// Whether a successful server version probe satisfies Testkit's protocol policy.
///
/// Stable 16.x servers are supported, matching the selected PocketIC client's
/// `>=16.0.0,<17.0.0` protocol range. Provisioning selects one exact reviewed
/// artifact separately; this predicate does not authenticate executable bytes.
#[must_use]
pub fn supports_pocket_ic_server(output: &[u8]) -> bool {
    let Ok(text) = std::str::from_utf8(output) else {
        return false;
    };
    let Some(version) = text.trim().strip_prefix("pocket-ic-server ") else {
        return false;
    };
    let components = version.split('.').collect::<Vec<_>>();
    components.len() == 3
        && components[0] == "16"
        && components.iter().all(|part| {
            !part.is_empty()
                && (part.len() == 1 || !part.starts_with('0'))
                && part.bytes().all(|byte| byte.is_ascii_digit())
                && part.parse::<u64>().is_ok()
        })
}

#[cfg(test)]
mod tests {
    #[test]
    fn protocol_admission_is_independent_of_download_selection() {
        for version in ["16.0.0", "16.0.1", "16.1.0", "16.2.3"] {
            assert!(super::supports_pocket_ic_server(
                format!("pocket-ic-server {version}\n").as_bytes()
            ));
        }
        for output in [
            "pocket-ic-server 15.0.0",
            "pocket-ic-server 17.0.0",
            "pocket-ic-server 16.0",
            "pocket-ic-server 16.01.0",
            "pocket-ic-server 16.1.0-extra",
            "other 16.1.0",
        ] {
            assert!(!super::supports_pocket_ic_server(output.as_bytes()));
        }
        assert!(!super::supports_pocket_ic_server(b"\xff"));
    }
}
