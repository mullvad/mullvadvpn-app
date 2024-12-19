package net.mullvad.mullvadvpn.repository

import androidx.datastore.core.DataStore
import co.touchlab.kermit.Logger
import java.io.IOException
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.catch
import kotlinx.coroutines.flow.first

class UserPreferencesRepository(private val userPreferences: DataStore<UserPreferences>) {

    // Note: this should not be made into a StateFlow. See:
    // https://developer.android.com/reference/kotlin/androidx/datastore/core/DataStore#data()
    val preferencesFlow: Flow<UserPreferences> =
        userPreferences.data.catch { exception ->
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
        userPreferences.updateData { prefs ->
            prefs.toBuilder().setIsPrivacyDisclosureAccepted(true).build()
        }
    }
}
