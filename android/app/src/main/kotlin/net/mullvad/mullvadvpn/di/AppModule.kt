package net.mullvad.mullvadvpn.di

import kotlinx.coroutines.MainScope
import net.mullvad.mullvadvpn.BuildConfig
import net.mullvad.mullvadvpn.constant.GRPC_SOCKET_FILE_NAME
import net.mullvad.mullvadvpn.lib.daemon.grpc.ManagementService
import net.mullvad.mullvadvpn.lib.intent.IntentProvider
import net.mullvad.mullvadvpn.lib.model.BuildVersion
import net.mullvad.mullvadvpn.lib.shared.AccountRepository
import net.mullvad.mullvadvpn.lib.shared.ConnectionProxy
import net.mullvad.mullvadvpn.lib.shared.VpnPermissionRepository
import org.koin.android.ext.koin.androidContext
import org.koin.core.qualifier.named
import org.koin.dsl.module

val appModule = module {
    single(named(RPC_SOCKET_PATH)) { "${androidContext().dataDir.path}/$GRPC_SOCKET_FILE_NAME" }
    single {
        ManagementService(
            rpcSocketPath = get(named(RPC_SOCKET_PATH)),
            extensiveLogging = BuildConfig.DEBUG,
            scope = MainScope(),
        )
    }
    single { BuildVersion(BuildConfig.VERSION_NAME, BuildConfig.VERSION_CODE) }
    single { IntentProvider() }
    single { AccountRepository(get(), MainScope()) }
    single { VpnPermissionRepository(androidContext()) }
    single { ConnectionProxy(get(), get()) }
}

const val RPC_SOCKET_PATH = "RPC_SOCKET"
