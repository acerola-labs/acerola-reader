use std::path::PathBuf;

use acerola_p2p::api::transport::{RelayModeConfig, ACEROLA_DEFAULT_RELAY_URL};
use serde_json::Value;
use tauri::Manager;
use tauri_plugin_fs::FsExt;

/// Extracts the `library_path` property from the settings JSON content.
fn extract_library_path(file_content: &str) -> Option<PathBuf> {
    let json_value: Value = serde_json::from_str(file_content).ok()?;
    let path_str = json_value.get("library_path")?.as_str()?;
    Some(PathBuf::from(path_str))
}

/// Reads `library_path` straight from `settings.json`, without depending on the store plugin
/// being initialized. Used both by the FS scope setup and by P2P file sync, which needs to
/// know where to write chapters received from another device.
pub fn read_library_path(app_data_directory: &std::path::Path) -> Option<PathBuf> {
    let settings_file_path = app_data_directory.join("settings.json");
    let file_content = std::fs::read_to_string(&settings_file_path).ok()?;
    extract_library_path(&file_content)
}

/// Relay configuration resolved from `settings.json`, combinable across multiple
/// simultaneous sources (see [`RelaySettings::resolve`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelaySettings {
    /// Uses the official relay maintained by Acerola (`ACEROLA_DEFAULT_RELAY_URL`).
    pub use_acerola_relay: bool,
    /// Uses the Iroh project's (n0) default public relay network — mutually exclusive with
    /// the other sources: `iroh::RelayMode` only allows `Disabled | Default | Custom`, never
    /// a combination of `Default` with specific URLs.
    pub use_iroh_public_network: bool,
    /// The user's own relay(s) (self-hosted, no authentication).
    pub custom_relay_urls: Vec<String>,
}

impl Default for RelaySettings {
    fn default() -> Self {
        Self {
            use_acerola_relay: true,
            use_iroh_public_network: false,
            custom_relay_urls: Vec::new(),
        }
    }
}

impl RelaySettings {
    /// Resolves the enabled sources into the concrete `RelayModeConfig` consumed by the
    /// `IrohTransportBuilder`. With no active source it falls back to `MdnsOnly` — there is no
    /// explicit "mDNS only" mode in the UI, it is just the natural "nothing selected" state.
    ///
    /// `iroh_services_ticket` comes from the encrypted vault (`SecureP2pStorage`), not from
    /// `settings.json` — it is the user's OWN account on `services.iroh.computer`, never a
    /// project secret embedded in the build. If the toggle is on but no ticket has been pasted
    /// yet, that source is ignored (there is no way to build `RelayModeConfig::IrohDefault`
    /// without it) and the other sources combine normally.
    pub fn resolve(&self, iroh_services_ticket: Option<&str>) -> RelayModeConfig {
        if self.use_iroh_public_network {
            if let Some(ticket) = iroh_services_ticket {
                return RelayModeConfig::IrohDefault(ticket.to_string());
            }
        }

        let mut urls = Vec::new();
        if self.use_acerola_relay {
            urls.push(ACEROLA_DEFAULT_RELAY_URL.to_string());
        }
        urls.extend(self.custom_relay_urls.iter().cloned());

        if urls.is_empty() {
            RelayModeConfig::MdnsOnly
        } else {
            RelayModeConfig::Custom(urls)
        }
    }
}

/// Reads the combinable relay configuration from `settings.json` (`relay_use_acerola`,
/// `relay_use_iroh_public`, `relay_custom_urls`). A missing file/keys falls back to
/// [`RelaySettings::default`] (Acerola relay enabled, no other source). Only read at startup:
/// changing the relay configuration at runtime is not supported by the lib, only switching
/// between local/relay mode (already exposed via `switch_to_local`/`switch_to_relay`).
///
/// Migration: if none of the new keys is present but the legacy `relay_url` key (from before
/// this feature, a single URL) is, it is migrated into `custom_relay_urls` — without this,
/// anyone who had configured their own relay would silently lose that choice on the first boot
/// after the update. `relay_iroh_urls` (a separate "Iroh Relays" list, removed for being
/// functionally identical to `relay_custom_urls` — both became the same
/// `RelayModeConfig::Custom`) is always merged into `custom_relay_urls` if present, even
/// when the new keys already exist.
pub fn read_relay_settings(app_data_directory: &std::path::Path) -> RelaySettings {
    let settings_file_path = app_data_directory.join("settings.json");
    let Ok(file_content) = std::fs::read_to_string(&settings_file_path) else {
        return RelaySettings::default();
    };
    let Ok(json_value) = serde_json::from_str::<Value>(&file_content) else {
        return RelaySettings::default();
    };

    let legacy_iroh_urls = read_string_array(&json_value, "relay_iroh_urls");

    if !has_any_relay_settings_key(&json_value) {
        if let Some(legacy_url) = read_legacy_relay_url(&json_value) {
            tracing::info!(
                "[Bios::Scopes] Migrating legacy 'relay_url' override into 'relay_custom_urls'"
            );
            let mut custom_relay_urls = vec![legacy_url];
            custom_relay_urls.extend(legacy_iroh_urls);
            return RelaySettings { custom_relay_urls, ..RelaySettings::default() };
        }
        if legacy_iroh_urls.is_empty() {
            return RelaySettings::default();
        }
        return RelaySettings { custom_relay_urls: legacy_iroh_urls, ..RelaySettings::default() };
    }

    let mut custom_relay_urls = read_string_array(&json_value, "relay_custom_urls");
    for url in legacy_iroh_urls {
        if !custom_relay_urls.contains(&url) {
            custom_relay_urls.push(url);
        }
    }

    RelaySettings {
        use_acerola_relay: json_value
            .get("relay_use_acerola")
            .and_then(Value::as_bool)
            .unwrap_or(true),
        use_iroh_public_network: json_value
            .get("relay_use_iroh_public")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        custom_relay_urls,
    }
}

fn has_any_relay_settings_key(json_value: &Value) -> bool {
    ["relay_use_acerola", "relay_use_iroh_public", "relay_custom_urls", "relay_iroh_urls"]
        .iter()
        .any(|key| json_value.get(key).is_some())
}

fn read_legacy_relay_url(json_value: &Value) -> Option<String> {
    let relay_url = json_value.get("relay_url")?.as_str()?.trim().to_string();
    if relay_url.is_empty() {
        return None;
    }
    Some(relay_url)
}

fn read_string_array(json_value: &Value, key: &str) -> Vec<String> {
    json_value
        .get(key)
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(str::trim)
                .filter(|url| !url.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

/// Reads the local device's custom alias (`device_alias`, LocalSend style) from
/// `settings.json`. Missing/empty (the common case) makes the caller fall back to the
/// `DeviceInfoProvider`'s automatic hostname. Only read when the P2P node starts — after that,
/// an alias set at runtime (`set_local_device_name`) no longer goes through here, it is only
/// persisted in this key by the frontend to survive the next restart.
pub fn read_device_alias_override(app_data_directory: &std::path::Path) -> Option<String> {
    let settings_file_path = app_data_directory.join("settings.json");
    let file_content = std::fs::read_to_string(&settings_file_path).ok()?;
    let json_value: Value = serde_json::from_str(&file_content).ok()?;
    let alias = json_value.get("device_alias")?.as_str()?.trim().to_string();

    if alias.is_empty() {
        return None;
    }

    Some(alias)
}

/// Registers the filesystem access permissions in Tauri using Early Returns (Guard Clauses)
/// to keep cyclomatic complexity minimal and the code linear.
pub async fn setup_scopes_from_store<R: tauri::Runtime>(
    app_handle: &tauri::AppHandle<R>, app_data_directory: &std::path::Path,
) {
    let library_path = match read_library_path(app_data_directory) {
        Some(path) => path,
        None => {
            tracing::warn!("[Bios::Scopes] Key 'library_path' missing or invalid in settings.json");
            return;
        },
    };

    apply_library_scope(app_handle, &library_path);
}

fn apply_library_scope<R: tauri::Runtime>(
    app_handle: &tauri::AppHandle<R>, library_path: &std::path::Path,
) {
    tracing::info!("[Bios::Scopes] Registering filesystem scope for {:?}", library_path);

    if let Err(scope_error) = app_handle.fs_scope().allow_directory(library_path, true) {
        tracing::error!("[Bios::Scopes] Failed to allow directory in fs_scope: {}", scope_error);
    }

    if let Err(scope_error) = app_handle.asset_protocol_scope().allow_directory(library_path, true)
    {
        tracing::error!(
            "[Bios::Scopes] Failed to allow directory in asset_protocol_scope: {}",
            scope_error
        );
    }
}

#[cfg(test)]
mod tests {
    use acerola_p2p::api::transport::RelayModeConfig;
    use tauri::Manager;
    use tauri_plugin_fs::FsExt;

    use super::{
        apply_library_scope, extract_library_path, read_device_alias_override, read_library_path,
        read_relay_settings, RelaySettings,
    };

    #[test]
    fn test_extract_library_path_returns_the_configured_path() {
        let content = r#"{"library_path":"/home/user/comics"}"#;
        assert_eq!(
            extract_library_path(content),
            Some(std::path::PathBuf::from("/home/user/comics"))
        );
    }

    #[test]
    fn test_extract_library_path_missing_key_returns_none() {
        assert_eq!(extract_library_path(r#"{"other_key":"value"}"#), None);
    }

    #[test]
    fn test_read_library_path_returns_the_configured_path() {
        let app_data_directory = tempfile::tempdir().unwrap();
        std::fs::write(
            app_data_directory.path().join("settings.json"),
            r#"{"library_path":"/library/comics"}"#,
        )
        .unwrap();

        assert_eq!(
            read_library_path(app_data_directory.path()),
            Some(std::path::PathBuf::from("/library/comics"))
        );
    }

    #[test]
    fn test_read_relay_settings_missing_file_returns_default() {
        let app_data_directory = tempfile::tempdir().unwrap();
        assert_eq!(read_relay_settings(app_data_directory.path()), RelaySettings::default());
    }

    #[test]
    fn test_read_relay_settings_reads_all_configured_fields() {
        let app_data_directory = tempfile::tempdir().unwrap();
        std::fs::write(
            app_data_directory.path().join("settings.json"),
            r#"{
                "relay_use_acerola": false,
                "relay_use_iroh_public": false,
                "relay_custom_urls": ["https://relay-a.test.local", "  ", "https://relay-b.test.local"]
            }"#,
        )
        .unwrap();

        assert_eq!(
            read_relay_settings(app_data_directory.path()),
            RelaySettings {
                use_acerola_relay: false,
                use_iroh_public_network: false,
                custom_relay_urls: vec![
                    "https://relay-a.test.local".to_string(),
                    "https://relay-b.test.local".to_string()
                ],
            }
        );
    }

    /// `relay_iroh_urls` was a separate list ("Iroh Relays") functionally identical to
    /// `relay_custom_urls` (both became the same `RelayModeConfig::Custom`) — removed from the
    /// UI, but anyone who already had something saved there must not silently lose that URL.
    #[test]
    fn test_read_relay_settings_merges_legacy_iroh_urls_into_custom_urls() {
        let app_data_directory = tempfile::tempdir().unwrap();
        std::fs::write(
            app_data_directory.path().join("settings.json"),
            r#"{
                "relay_custom_urls": ["https://relay-a.test.local"],
                "relay_iroh_urls": ["https://iroh-relay.test.local"]
            }"#,
        )
        .unwrap();

        assert_eq!(
            read_relay_settings(app_data_directory.path()),
            RelaySettings {
                custom_relay_urls: vec![
                    "https://relay-a.test.local".to_string(),
                    "https://iroh-relay.test.local".to_string()
                ],
                ..RelaySettings::default()
            }
        );
    }

    #[test]
    fn test_read_relay_settings_migrates_legacy_iroh_urls_only_when_no_other_key_present() {
        let app_data_directory = tempfile::tempdir().unwrap();
        std::fs::write(
            app_data_directory.path().join("settings.json"),
            r#"{"relay_iroh_urls": ["https://iroh-relay.test.local"]}"#,
        )
        .unwrap();

        assert_eq!(
            read_relay_settings(app_data_directory.path()),
            RelaySettings {
                custom_relay_urls: vec!["https://iroh-relay.test.local".to_string()],
                ..RelaySettings::default()
            }
        );
    }

    #[test]
    fn test_read_relay_settings_does_not_duplicate_url_present_in_both_lists() {
        let app_data_directory = tempfile::tempdir().unwrap();
        std::fs::write(
            app_data_directory.path().join("settings.json"),
            r#"{
                "relay_custom_urls": ["https://relay-a.test.local"],
                "relay_iroh_urls": ["https://relay-a.test.local"]
            }"#,
        )
        .unwrap();

        assert_eq!(
            read_relay_settings(app_data_directory.path()),
            RelaySettings {
                custom_relay_urls: vec!["https://relay-a.test.local".to_string()],
                ..RelaySettings::default()
            }
        );
    }

    #[test]
    fn test_read_relay_settings_migrates_legacy_relay_url_into_custom_urls() {
        let app_data_directory = tempfile::tempdir().unwrap();
        std::fs::write(
            app_data_directory.path().join("settings.json"),
            r#"{"relay_url":"https://relay.example.com"}"#,
        )
        .unwrap();

        assert_eq!(
            read_relay_settings(app_data_directory.path()),
            RelaySettings {
                custom_relay_urls: vec!["https://relay.example.com".to_string()],
                ..RelaySettings::default()
            }
        );
    }

    #[test]
    fn test_read_relay_settings_empty_legacy_relay_url_returns_default() {
        let app_data_directory = tempfile::tempdir().unwrap();
        std::fs::write(app_data_directory.path().join("settings.json"), r#"{"relay_url":"  "}"#)
            .unwrap();

        assert_eq!(read_relay_settings(app_data_directory.path()), RelaySettings::default());
    }

    #[test]
    fn test_relay_settings_resolve_nothing_active_returns_mdns_only() {
        let settings = RelaySettings {
            use_acerola_relay: false,
            use_iroh_public_network: false,
            custom_relay_urls: vec![],
        };
        assert_eq!(settings.resolve(None), RelayModeConfig::MdnsOnly);
    }

    #[test]
    fn test_relay_settings_resolve_merges_all_active_sources_into_custom() {
        let settings = RelaySettings {
            use_acerola_relay: true,
            use_iroh_public_network: false,
            custom_relay_urls: vec!["https://relay-a.test.local".to_string()],
        };

        assert_eq!(
            settings.resolve(None),
            RelayModeConfig::Custom(vec![
                acerola_p2p::api::transport::ACEROLA_DEFAULT_RELAY_URL.to_string(),
                "https://relay-a.test.local".to_string(),
            ])
        );
    }

    #[test]
    fn test_relay_settings_resolve_iroh_public_network_with_ticket_ignores_other_sources() {
        let settings = RelaySettings {
            use_acerola_relay: true,
            use_iroh_public_network: true,
            custom_relay_urls: vec!["https://relay-a.test.local".to_string()],
        };

        assert_eq!(
            settings.resolve(Some("services-fake-ticket")),
            RelayModeConfig::IrohDefault("services-fake-ticket".to_string())
        );
    }

    /// Toggle on but no ticket pasted yet (first use, or the user unchecked it after having
    /// saved to `settings.json`) — there is no way to build `IrohDefault` without the ticket, so
    /// it falls back to the other sources combined normally, as if the toggle were off.
    #[test]
    fn test_relay_settings_resolve_iroh_public_network_without_ticket_falls_back_to_other_sources()
    {
        let settings = RelaySettings {
            use_acerola_relay: true,
            use_iroh_public_network: true,
            custom_relay_urls: vec![],
        };

        assert_eq!(
            settings.resolve(None),
            RelayModeConfig::Custom(vec![
                acerola_p2p::api::transport::ACEROLA_DEFAULT_RELAY_URL.to_string()
            ])
        );
    }

    #[test]
    fn test_read_device_alias_override_returns_the_configured_value() {
        let app_data_directory = tempfile::tempdir().unwrap();
        std::fs::write(
            app_data_directory.path().join("settings.json"),
            r#"{"device_alias":"Notebook do Vinicius"}"#,
        )
        .unwrap();

        assert_eq!(
            read_device_alias_override(app_data_directory.path()),
            Some("Notebook do Vinicius".to_string())
        );
    }

    #[test]
    fn test_read_device_alias_override_missing_key_returns_none() {
        let app_data_directory = tempfile::tempdir().unwrap();
        std::fs::write(app_data_directory.path().join("settings.json"), r#"{}"#).unwrap();

        assert_eq!(read_device_alias_override(app_data_directory.path()), None);
    }

    #[test]
    fn test_read_device_alias_override_empty_value_returns_none() {
        let app_data_directory = tempfile::tempdir().unwrap();
        std::fs::write(
            app_data_directory.path().join("settings.json"),
            r#"{"device_alias":"   "}"#,
        )
        .unwrap();

        assert_eq!(read_device_alias_override(app_data_directory.path()), None);
    }

    fn build_mock_app() -> tauri::App<tauri::test::MockRuntime> {
        tauri::test::mock_builder()
            .plugin(tauri_plugin_fs::init())
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .unwrap()
    }

    #[test]
    fn test_apply_library_scope_allows_the_directory() {
        let app = build_mock_app();
        let app_handle = app.handle();
        let library_directory = tempfile::tempdir().unwrap();

        apply_library_scope(app_handle, library_directory.path());

        assert!(app_handle.fs_scope().is_allowed(library_directory.path()));
        assert!(app_handle.asset_protocol_scope().is_allowed(library_directory.path()));
    }

    #[test]
    fn test_setup_scopes_from_store_registers_scope_from_settings() {
        let app = build_mock_app();
        let app_handle = app.handle();

        let app_data_directory = tempfile::tempdir().unwrap();
        let library_directory = tempfile::tempdir().unwrap();
        std::fs::write(
            app_data_directory.path().join("settings.json"),
            format!(r#"{{"library_path":{:?}}}"#, library_directory.path()),
        )
        .unwrap();

        tauri::async_runtime::block_on(super::setup_scopes_from_store(
            app_handle,
            app_data_directory.path(),
        ));

        assert!(app_handle.fs_scope().is_allowed(library_directory.path()));
        assert!(app_handle.asset_protocol_scope().is_allowed(library_directory.path()));
    }
}
