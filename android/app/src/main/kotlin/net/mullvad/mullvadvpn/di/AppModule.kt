package net.mullvad.mullvadvpn.di

import kotlinx.coroutines.MainScope
import net.mullvad.mullvadvpn.constant.GRPC_SOCKET_PATH
import net.mullvad.mullvadvpn.lib.account.AccountRepository
import net.mullvad.mullvadvpn.lib.daemon.grpc.ManagementService
import net.mullvad.mullvadvpn.lib.intent.BuildConfig
import net.mullvad.mullvadvpn.lib.intent.IntentProvider
import net.mullvad.mullvadvpn.lib.permission.VpnPermissionRepository
import org.koin.android.ext.koin.androidContext
import org.koin.dsl.module

val appModule = module {
    single {
        ManagementService(
            rpcSocketPath = GRPC_SOCKET_PATH,
            interceptLogging = BuildConfig.DEBUG,
            scope = MainScope(),
        )
    }
    single { IntentProvider() }
    single { AccountRepository(get(), MainScope()) }
    single { VpnPermissionRepository(androidContext()) }
}
