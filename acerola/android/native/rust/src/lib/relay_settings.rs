use acerola_p2p::api::transport::{RelayModeConfig, ACEROLA_DEFAULT_RELAY_URL};

/// Combinable relay configuration exposed to the FFI boundary (Kotlin) — mirrors Desktop's
/// `RelaySettings`/`RelaySettings::resolve` (`bios/scopes.rs`). The only difference is
/// where the sources come from: on Desktop Rust itself reads them from `settings.json`, on Android
/// Kotlin reads them from DataStore (`RelayPreference`) and passes them whole here when building the
/// `P2PNode` — the logic combining the sources into a `RelayModeConfig` is the same in both apps.
#[derive(uniffi::Record, Clone, Debug, PartialEq, Eq)]
pub struct FfiRelaySettings {
    /// Uses the official relay maintained by Acerola (`ACEROLA_DEFAULT_RELAY_URL`).
    pub use_acerola_relay: bool,
    /// Uses the Iroh project's (n0) default public relay network — mutually exclusive with the
    /// other sources: `iroh::RelayMode` only allows `Disabled | Default | Custom`, never a
    /// combination of `Default` with specific URLs.
    pub use_iroh_public_network: bool,
    /// The user's own relay(s) (self-hosted, no authentication).
    pub custom_relay_urls: Vec<String>,
}

/// Error exposed to the FFI boundary (Kotlin) when the user pastes a malformed ticket —
/// validated before persisting, instead of just silently failing on the next boot.
#[derive(uniffi::Error, thiserror::Error, Debug)]
pub enum RelayTicketError {
    #[error("invalid Iroh Services ticket: {reason}")]
    Invalid { reason: String },
}

impl FfiRelaySettings {
    /// Resolves the enabled sources into the concrete `RelayModeConfig` consumed by
    /// `IrohTransportBuilder`. With no active source, falls back to `MdnsOnly` — there is no explicit
    /// "mDNS only" mode in the UI, it is just the natural "nothing selected" state.
    ///
    /// `iroh_services_ticket` comes from the encrypted vault (`SecureP2pStorage`), not from
    /// DataStore — it is the user's OWN account on `services.iroh.computer`, a real
    /// credential, not just any preference. If the toggle is on but no ticket has
    /// been pasted yet, that source is ignored and the others combine normally.
    pub(crate) fn resolve(&self, iroh_services_ticket: Option<&str>) -> RelayModeConfig {
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

#[cfg(test)]
mod tests {
    use acerola_p2p::api::transport::{RelayModeConfig, ACEROLA_DEFAULT_RELAY_URL};

    use super::FfiRelaySettings;

    #[test]
    fn test_resolve_nothing_active_returns_mdns_only() {
        let settings = FfiRelaySettings {
            use_acerola_relay: false,
            use_iroh_public_network: false,
            custom_relay_urls: vec![],
        };

        assert_eq!(settings.resolve(None), RelayModeConfig::MdnsOnly);
    }

    #[test]
    fn test_resolve_merges_all_active_sources_into_custom() {
        let settings = FfiRelaySettings {
            use_acerola_relay: true,
            use_iroh_public_network: false,
            custom_relay_urls: vec!["https://relay-a.test.local".to_string()],
        };

        assert_eq!(
            settings.resolve(None),
            RelayModeConfig::Custom(vec![
                ACEROLA_DEFAULT_RELAY_URL.to_string(),
                "https://relay-a.test.local".to_string(),
            ])
        );
    }

    #[test]
    fn test_resolve_iroh_public_network_with_ticket_ignores_other_sources() {
        let settings = FfiRelaySettings {
            use_acerola_relay: true,
            use_iroh_public_network: true,
            custom_relay_urls: vec!["https://relay-a.test.local".to_string()],
        };

        assert_eq!(
            settings.resolve(Some("services-fake-ticket")),
            RelayModeConfig::IrohDefault("services-fake-ticket".to_string())
        );
    }

    #[test]
    fn test_resolve_iroh_public_network_without_ticket_falls_back_to_other_sources() {
        let settings = FfiRelaySettings {
            use_acerola_relay: true,
            use_iroh_public_network: true,
            custom_relay_urls: vec![],
        };

        assert_eq!(
            settings.resolve(None),
            RelayModeConfig::Custom(vec![ACEROLA_DEFAULT_RELAY_URL.to_string()])
        );
    }

    #[test]
    fn test_resolve_only_acerola_relay_active() {
        let settings = FfiRelaySettings {
            use_acerola_relay: true,
            use_iroh_public_network: false,
            custom_relay_urls: vec![],
        };

        assert_eq!(
            settings.resolve(None),
            RelayModeConfig::Custom(vec![ACEROLA_DEFAULT_RELAY_URL.to_string()])
        );
    }
}
