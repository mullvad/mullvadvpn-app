package net.mullvad.mullvadvpn.lib.model

import android.content.Context
import androidx.datastore.core.DataStore
import androidx.datastore.dataStore
import net.mullvad.mullvadvpn.model.TunnelPreference
import org.koin.android.ext.koin.androidContext
import org.koin.dsl.module

val modelModule = module {
    single<DataStore<TunnelPreference>> { androidContext().tunnelPreferencesStore }
    single<TunnelPreferencesRepository> { TunnelPreferencesRepository(get()) }
}

private val Context.tunnelPreferencesStore: DataStore<TunnelPreference> by
    dataStore(
        fileName = "net.mullvad.mullvadvpn_tunnel_preferences",
        serializer = TunnelPreferencesSerializer,
    )
