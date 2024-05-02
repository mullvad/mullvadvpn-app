package net.mullvad.mullvadvpn.di

import kotlinx.coroutines.MainScope
import net.mullvad.mullvadvpn.lib.daemon.grpc.ManagementService
import net.mullvad.mullvadvpn.lib.intent.IntentProvider
import net.mullvad.mullvadvpn.repository.MigrateSplitTunnelingRepository
import net.mullvad.mullvadvpn.repository.SplitTunnelingRepository
import org.koin.android.ext.koin.androidContext
import org.koin.dsl.module

val appModule = module {
    single {
        ManagementService(
            "/data/data/net.mullvad.mullvadvpn/rpc-socket",
            MainScope(),
        )
    }
    single { IntentProvider() }
}
