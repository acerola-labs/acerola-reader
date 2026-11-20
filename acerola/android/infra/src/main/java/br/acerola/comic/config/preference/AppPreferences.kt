package br.acerola.comic.config.preference

import android.content.Context
import androidx.datastore.preferences.core.Preferences
import androidx.datastore.preferences.core.booleanPreferencesKey
import androidx.datastore.preferences.core.edit
import androidx.datastore.preferences.core.stringPreferencesKey
import androidx.datastore.preferences.core.stringSetPreferencesKey
import androidx.datastore.preferences.preferencesDataStore
import br.acerola.comic.config.preference.types.VolumeViewType
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.map

object VolumeViewPreference {
    private val Context.dataStore by preferencesDataStore(name = "volume_view_prefs")

    private val VOLUME_VIEW_KEY = stringPreferencesKey(name = "volume_view_type")

    suspend fun saveVolumeView(
        context: Context,
        mode: VolumeViewType,
    ) {
        context.dataStore.edit { prefs ->
            prefs[VOLUME_VIEW_KEY] = mode.key
        }
    }

    fun volumeViewFlow(context: Context): Flow<VolumeViewType> =
        context.dataStore.data.map { prefs ->
            val saved = VolumeViewType.fromKey(prefs[VOLUME_VIEW_KEY])
            if (saved == VolumeViewType.CHAPTER) VolumeViewType.VOLUME else saved
        }
}

object MetadataPreference {
    private val Context.dataStore by preferencesDataStore(name = "metadata_prefs")
    private val GENERATE_COMIC_INFO_KEY = booleanPreferencesKey(name = "generate_comic_info_by_default")
    private val METADATA_LANGUAGE_KEY = stringPreferencesKey(name = "metadata_language")

    suspend fun saveGenerateComicInfo(
        context: Context,
        generate: Boolean,
    ) {
        context.dataStore.edit { prefs ->
            prefs[GENERATE_COMIC_INFO_KEY] = generate
        }
    }

    suspend fun saveMetadataLanguage(
        context: Context,
        language: String,
    ) {
        context.dataStore.edit { prefs ->
            prefs[METADATA_LANGUAGE_KEY] = language
        }
    }

    fun generateComicInfoFlow(context: Context): Flow<Boolean> =
        context.dataStore.data.map { prefs ->
            prefs[GENERATE_COMIC_INFO_KEY] ?: true
        }

    fun metadataLanguageFlow(context: Context): Flow<String?> =
        context.dataStore.data.map { prefs ->
            prefs[METADATA_LANGUAGE_KEY]
        }
}

object ComicDirectoryPreference {
    private val Context.dataStore by preferencesDataStore(name = "folder_prefs")
    private val FOLDER_URI = stringPreferencesKey(name = "folder_uri")
    private val TUTORIAL_SHOWN = booleanPreferencesKey(name = "tutorial_shown")

    suspend fun saveFolderUri(
        context: Context,
        uri: String,
    ) {
        context.dataStore.edit { prefs ->
            prefs[FOLDER_URI] = uri
        }
    }

    suspend fun clearFolderUri(context: Context) {
        context.dataStore.edit { prefs ->
            prefs.remove(key = FOLDER_URI)
        }
    }

    fun folderUriFlow(context: Context): Flow<String?> = context.dataStore.data.map { prefs -> prefs[FOLDER_URI] }

    suspend fun setTutorialShown(
        context: Context,
        shown: Boolean,
    ) {
        context.dataStore.edit { prefs ->
            prefs[TUTORIAL_SHOWN] = shown
        }
    }

    fun tutorialShownFlow(context: Context): Flow<Boolean> = context.dataStore.data.map { prefs -> prefs[TUTORIAL_SHOWN] ?: false }
}

/**
 * `true` when the user chose to skip the "sync using mobile data" confirmation
 * (see `SyncViewModel`/`NetworkConnectivity.isOnCellularConnection`) — default `false`: by
 * default the app always asks before triggering a P2P sync on the cellular network.
 */
object MobileDataSyncPreference {
    private val Context.dataStore by preferencesDataStore(name = "mobile_data_sync_prefs")
    private val ALWAYS_ALLOW = booleanPreferencesKey(name = "always_allow_mobile_data_sync")

    suspend fun setAlwaysAllow(
        context: Context,
        value: Boolean,
    ) {
        context.dataStore.edit { prefs -> prefs[ALWAYS_ALLOW] = value }
    }

    fun alwaysAllowFlow(context: Context): Flow<Boolean> = context.dataStore.data.map { prefs -> prefs[ALWAYS_ALLOW] ?: false }
}

object DeviceAliasPreference {
    private val Context.dataStore by preferencesDataStore(name = "device_alias_prefs")
    private val DEVICE_ALIAS = stringPreferencesKey(name = "device_alias")

    suspend fun saveAlias(
        context: Context,
        name: String,
    ) {
        context.dataStore.edit { prefs ->
            prefs[DEVICE_ALIAS] = name
        }
    }

    /** `null` when the user never set a nickname — the caller should fall back to the
     *  automatic name (`Build.MODEL`). */
    fun deviceAliasFlow(context: Context): Flow<String?> = context.dataStore.data.map { prefs -> prefs[DEVICE_ALIAS] }
}

/**
 * Local nickname per paired peer (contact book style) — only exists on this device,
 * never exchanged over the P2P protocol. It sits on top of the default `deviceName` the peer announces
 * in the handshake, never overwriting it: the reader decides the display priority (see
 * `SyncScreen::PeerRow`). One key per peer (`peer_nickname_<peerId>`) instead of a single serialized
 * value — Preferences DataStore has no native Map type, and this avoids depending on
 * JSON serialization just for this.
 */
object PeerNicknamePreference {
    private val Context.dataStore by preferencesDataStore(name = "peer_nickname_prefs")
    private const val KEY_PREFIX = "peer_nickname_"

    private fun keyFor(peerId: String) = stringPreferencesKey(name = "$KEY_PREFIX$peerId")

    /** A null or blank `nickname` clears the saved nickname (falls back to the default
     *  `deviceName` again). */
    suspend fun setNickname(
        context: Context,
        peerId: String,
        nickname: String?,
    ) {
        context.dataStore.edit { prefs ->
            val trimmed = nickname?.trim()
            if (trimmed.isNullOrEmpty()) {
                prefs.remove(keyFor(peerId))
            } else {
                prefs[keyFor(peerId)] = trimmed
            }
        }
    }

    /** Map of peerId -> saved local nickname. */
    fun nicknamesFlow(context: Context): Flow<Map<String, String>> =
        context.dataStore.data.map { prefs ->
            prefs
                .asMap()
                .entries
                .filter { (key, _) -> key.name.startsWith(KEY_PREFIX) }
                .associate { (key, value) -> key.name.removePrefix(KEY_PREFIX) to value as String }
        }
}

/**
 * Combinable relay configuration, mirroring Desktop's `RelaySettings`/`RelaySettings::resolve`
 * (`bios/scopes.rs`) — the Acerola relay and the list of custom relays can be combined
 * with each other (except Iroh Services, exclusive with the others). Changing any source only takes
 * effect on the next app start: `P2pService`/`NetworkCaseModule` only reads this when building the
 * `P2PNode`, the lib does not support switching relay at runtime.
 */
object RelayPreference {
    const val DEFAULT_ACEROLA_RELAY_URL = "https://relay.acerola.app"

    private val Context.dataStore by preferencesDataStore(name = "relay_prefs")
    private val USE_ACEROLA_RELAY = booleanPreferencesKey(name = "use_acerola_relay")
    private val USE_IROH_PUBLIC_NETWORK = booleanPreferencesKey(name = "use_iroh_public_network")
    private val CUSTOM_RELAY_URLS = stringSetPreferencesKey(name = "custom_relay_urls")

    // Legacy key (from before this feature, single URL) — only read when none of the new
    // keys above exist yet, to migrate into `customRelayUrls`. Without this, anyone who had
    // already configured a custom relay would silently lose that choice on the first boot
    // after the update (same care Desktop takes with `relay_url` in `settings.json`).
    private val LEGACY_RELAY_URL_OVERRIDE = stringPreferencesKey(name = "relay_url_override")

    // "Iroh relays" was a separate list, functionally identical to `customRelayUrls` (both
    // became the same `RelayModeConfig::Custom`) — removed from the UI, but always merged here if
    // someone already had something saved, so the URL is not silently lost.
    private val LEGACY_IROH_RELAY_URLS = stringSetPreferencesKey(name = "iroh_relay_urls")

    data class RelaySettings(
        val useAcerolaRelay: Boolean = true,
        val useIrohPublicNetwork: Boolean = false,
        val customRelayUrls: List<String> = emptyList(),
    )

    suspend fun setUseAcerolaRelay(
        context: Context,
        value: Boolean,
    ) {
        context.dataStore.edit { prefs -> prefs[USE_ACEROLA_RELAY] = value }
    }

    suspend fun setUseIrohPublicNetwork(
        context: Context,
        value: Boolean,
    ) {
        context.dataStore.edit { prefs -> prefs[USE_IROH_PUBLIC_NETWORK] = value }
    }

    suspend fun addCustomRelayUrl(
        context: Context,
        url: String,
    ) {
        context.dataStore.edit { prefs -> prefs[CUSTOM_RELAY_URLS] = (prefs[CUSTOM_RELAY_URLS] ?: emptySet()) + url }
    }

    suspend fun removeCustomRelayUrl(
        context: Context,
        url: String,
    ) {
        context.dataStore.edit { prefs -> prefs[CUSTOM_RELAY_URLS] = (prefs[CUSTOM_RELAY_URLS] ?: emptySet()) - url }
    }

    fun relaySettingsFlow(context: Context): Flow<RelaySettings> = context.dataStore.data.map(::toRelaySettings)

    /** One-off read (without collecting the `Flow`) — used in the synchronous `P2PNode` startup
     *  (see `NetworkCaseModule`), same pattern as the old `relayUrlOverrideFlow(...).first()`. */
    suspend fun currentRelaySettings(context: Context): RelaySettings = toRelaySettings(context.dataStore.data.first())

    private fun toRelaySettings(prefs: Preferences): RelaySettings {
        val legacyIrohRelayUrls = prefs[LEGACY_IROH_RELAY_URLS] ?: emptySet()
        val hasAnyNewKey =
            prefs.contains(USE_ACEROLA_RELAY) ||
                prefs.contains(USE_IROH_PUBLIC_NETWORK) ||
                prefs.contains(CUSTOM_RELAY_URLS)

        if (!hasAnyNewKey) {
            val legacyUrl = prefs[LEGACY_RELAY_URL_OVERRIDE]?.trim()
            val customRelayUrls =
                if (!legacyUrl.isNullOrEmpty()) {
                    (listOf(legacyUrl) + legacyIrohRelayUrls).distinct()
                } else {
                    legacyIrohRelayUrls.toList()
                }
            return RelaySettings(customRelayUrls = customRelayUrls)
        }

        val customRelayUrls = ((prefs[CUSTOM_RELAY_URLS] ?: emptySet()) + legacyIrohRelayUrls).distinct()
        return RelaySettings(
            useAcerolaRelay = prefs[USE_ACEROLA_RELAY] ?: true,
            useIrohPublicNetwork = prefs[USE_IROH_PUBLIC_NETWORK] ?: false,
            customRelayUrls = customRelayUrls,
        )
    }
}
