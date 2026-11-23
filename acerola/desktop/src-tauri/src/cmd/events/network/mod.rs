use std::collections::HashSet;

use acerola_p2p::api::{
    identity::DeviceInfo, network::NetworkMode, peer, transport::ACEROLA_DEFAULT_RELAY_URL,
};
use serde::{Deserialize, Serialize};

use crate::bios::scopes::RelaySettings;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInfoPayload {
    pub name: String,
    pub os: String,
    pub version: String,
}

impl From<DeviceInfo> for DeviceInfoPayload {
    fn from(d: DeviceInfo) -> Self {
        Self { name: d.name, os: d.os, version: d.version }
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConnectedPeerPayload {
    pub peer_id: String,
    pub alpn: String,
    pub device: Option<DeviceInfoPayload>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NetworkStatusPayload {
    pub mode: String,
    pub peers: Vec<ConnectedPeerPayload>,
}

impl NetworkStatusPayload {
    pub fn from(
        mode: NetworkMode, peers: Vec<(peer::PeerIdentity, HashSet<Vec<u8>>, Option<DeviceInfo>)>,
    ) -> Self {
        let mode_str = match mode {
            NetworkMode::Local => "local",
            NetworkMode::Relay => "relay",
        };

        let peer_list: Vec<ConnectedPeerPayload> = peers
            .into_iter()
            .flat_map(|(peer, alpns, device_info)| {
                let peer_id = peer.id;
                let device = device_info.map(DeviceInfoPayload::from);

                alpns.into_iter().map(move |alpn| ConnectedPeerPayload {
                    peer_id: peer_id.clone(),
                    alpn: String::from_utf8_lossy(&alpn).into_owned(),
                    device: device.clone(),
                })
            })
            .collect();

        Self { mode: mode_str.to_string(), peers: peer_list }
    }
}

/// Peer paired (TOFU) at some point, with the last known address to trigger `connect`.
/// `device_name` comes from `AcerolaP2p::known_peers()` (see `NetworkServiceApi::paired_peers`) —
/// it persists across restarts, unlike `ConnectedPeerPayload`'s `DeviceInfo` (which only exists
/// for the few seconds the handshake session is actually open). The frontend only falls back
/// to the raw id when this peer never answered the identity exchange.
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PairedPeerPayload {
    pub peer_id: String,
    pub addrs: Vec<u8>,
    pub device_name: Option<String>,
}

impl From<(peer::PeerAddr, Option<DeviceInfo>)> for PairedPeerPayload {
    fn from((addr, device_info): (peer::PeerAddr, Option<DeviceInfo>)) -> Self {
        Self { peer_id: addr.id.id, addrs: addr.addrs, device_name: device_info.map(|d| d.name) }
    }
}

/// Full relay configuration state, combining the sources read from
/// `settings.json` (see [`RelaySettings`]). Changing any field only takes effect on the
/// next app start, since the lib does not support changing the relay configuration at
/// runtime.
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RelayInfo {
    /// Fixed URL of Acerola's official relay, to display it even when disabled.
    pub acerola_relay_url: String,
    pub use_acerola_relay: bool,
    pub use_iroh_public_network: bool,
    pub custom_relay_urls: Vec<String>,
    /// Only tells WHETHER an Iroh Services ticket has been pasted and saved — the value itself
    /// never goes back to the frontend (it is a real credential, see `SecureP2pStorage::load_iroh_services_ticket`).
    pub has_iroh_services_ticket: bool,
}

impl RelayInfo {
    pub fn new(settings: RelaySettings, has_iroh_services_ticket: bool) -> Self {
        Self {
            acerola_relay_url: ACEROLA_DEFAULT_RELAY_URL.to_string(),
            use_acerola_relay: settings.use_acerola_relay,
            use_iroh_public_network: settings.use_iroh_public_network,
            custom_relay_urls: settings.custom_relay_urls,
            has_iroh_services_ticket,
        }
    }
}
