package net.mullvad.mullvadvpn.di

import kotlinx.coroutines.MainScope
import net.mullvad.mullvadvpn.constant.GRPC_SOCKET_FILE
import net.mullvad.mullvadvpn.lib.daemon.grpc.ManagementService
import net.mullvad.mullvadvpn.lib.intent.BuildConfig
import net.mullvad.mullvadvpn.lib.intent.IntentProvider
import net.mullvad.mullvadvpn.lib.shared.AccountRepository
import net.mullvad.mullvadvpn.lib.shared.ConnectionProxy
import net.mullvad.mullvadvpn.lib.shared.VpnPermissionRepository
import org.koin.android.ext.koin.androidContext
import org.koin.dsl.module

val appModule = module {
    single {
        ManagementService(
            rpcSocketPath = "${androidContext().dataDir.path}/$GRPC_SOCKET_FILE",
            extensiveLogging = BuildConfig.DEBUG,
            scope = MainScope(),
        )
    }
    single { IntentProvider() }
    single { AccountRepository(get(), MainScope()) }
    single { VpnPermissionRepository(androidContext()) }
    single { ConnectionProxy(get(), get()) }
}
