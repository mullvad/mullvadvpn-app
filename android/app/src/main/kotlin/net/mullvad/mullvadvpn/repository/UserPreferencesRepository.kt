package net.mullvad.mullvadvpn.repository

import androidx.datastore.core.DataStore
import java.time.Instant
import java.time.ZoneId
import java.time.ZonedDateTime
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.first
import net.mullvad.mullvadvpn.lib.model.BuildVersion

class UserPreferencesRepository(
    private val userPreferencesStore: DataStore<UserPreferences>,
    private val buildVersion: BuildVersion,
) {
    fun preferencesFlow(): Flow<UserPreferences> = userPreferencesStore.data

    suspend fun preferences(): UserPreferences = userPreferencesStore.data.first()

    suspend fun setPrivacyDisclosureAccepted() {
        userPreferencesStore.updateData { prefs ->
            prefs.toBuilder().setIsPrivacyDisclosureAccepted(true).build()
        }
    }

    suspend fun setHasDisplayedChangelogNotification() {
        userPreferencesStore.updateData { prefs ->
            prefs.toBuilder().setLastShownChangelogVersionCode(buildVersion.code).build()
        }
    }

    suspend fun setAccountExpiry(expiry: ZonedDateTime) {
        userPreferencesStore.updateData { prefs ->
            prefs.toBuilder().setAccountExpiryUnixTimeSeconds(expiry.toEpochSecond()).build()
        }
    }

    // Returns the account expiry time or null if the account expiry has not been set yet.
    suspend fun accountExpiry(): ZonedDateTime? =
        preferences().let { prefs ->
            val expiryTime = prefs.accountExpiryUnixTimeSeconds
            if (expiryTime == 0L) return null
            Instant.ofEpochSecond(expiryTime).atZone(ZoneId.systemDefault())
        }
}
