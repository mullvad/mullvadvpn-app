package net.mullvad.mullvadvpn.repository

import androidx.datastore.core.DataStore
import co.touchlab.kermit.Logger
import java.io.IOException
import java.time.Instant
import java.time.ZoneId
import java.time.ZonedDateTime
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.catch
import kotlinx.coroutines.flow.first
import net.mullvad.mullvadvpn.lib.model.BuildVersion

class UserPreferencesRepository(
    private val userPreferencesStore: DataStore<UserPreferences>,
    private val buildVersion: BuildVersion,
) {

    // Note: this should not be made into a StateFlow. See:
    // https://developer.android.com/reference/kotlin/androidx/datastore/core/DataStore#data()
    val preferencesFlow: Flow<UserPreferences> =
        userPreferencesStore.data.catch { exception ->
            // dataStore.data throws an IOException when an error is encountered when reading data
            if (exception is IOException) {
                Logger.e("Error reading user preferences file, falling back to default.", exception)
                emit(UserPreferences.getDefaultInstance())
            } else {
                throw exception
            }
        }

    suspend fun preferences(): UserPreferences = preferencesFlow.first()

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
