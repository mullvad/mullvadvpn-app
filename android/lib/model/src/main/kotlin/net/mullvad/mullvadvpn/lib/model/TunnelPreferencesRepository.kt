package net.mullvad.mullvadvpn.lib.model

import androidx.datastore.core.DataStore
import java.io.IOException
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.catch
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.runBlocking
import net.mullvad.mullvadvpn.model.TunnelPreference

class TunnelPreferencesRepository(private val userPreferencesStore: DataStore<TunnelPreference>) {

    // Note: this should not be made into a StateFlow. See:
    // https://developer.android.com/reference/kotlin/androidx/datastore/core/DataStore#data()
    val preferencesFlow: Flow<TunnelPreference> =
        userPreferencesStore.data.catch { exception ->
            // dataStore.data throws an IOException when an error is encountered when reading data
            if (exception is IOException) {
                // Logger.e("Error reading user preferences file, falling back to default.",
                // exception)
                emit(TunnelPreference.getDefaultInstance())
            } else {
                throw exception
            }
        }

    fun preferences(): TunnelPreference = runBlocking { preferencesFlow.first() }

    fun isRouteIpv6(): Boolean = preferences().routeIpV6

    suspend fun setRouteIpv6(enable: Boolean) {
        userPreferencesStore.updateData { prefs -> prefs.toBuilder().setRouteIpV6(enable).build() }
    }
}
