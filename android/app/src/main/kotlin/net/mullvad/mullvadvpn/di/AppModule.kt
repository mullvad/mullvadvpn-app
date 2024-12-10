package net.mullvad.mullvadvpn.di

import java.io.File
import kotlinx.coroutines.MainScope
import net.mullvad.mullvadvpn.BuildConfig
import net.mullvad.mullvadvpn.lib.common.constant.GRPC_SOCKET_FILE_NAME
import net.mullvad.mullvadvpn.lib.common.constant.GRPC_SOCKET_FILE_NAMED_ARGUMENT
import net.mullvad.mullvadvpn.lib.daemon.grpc.ManagementService
import net.mullvad.mullvadvpn.lib.endpoint.ApiEndpointFromIntentHolder
import net.mullvad.mullvadvpn.lib.model.BuildVersion
import net.mullvad.mullvadvpn.lib.shared.AccountRepository
import net.mullvad.mullvadvpn.lib.shared.ConnectionProxy
import net.mullvad.mullvadvpn.lib.shared.DeviceRepository
import net.mullvad.mullvadvpn.lib.shared.LocaleRepository
import net.mullvad.mullvadvpn.lib.shared.PrepareVpnUseCase
import net.mullvad.mullvadvpn.lib.shared.RelayLocationTranslationRepository
import org.koin.android.ext.koin.androidContext
import org.koin.core.qualifier.named
import org.koin.dsl.module

val appModule = module {
    single(named(GRPC_SOCKET_FILE_NAMED_ARGUMENT)) {
        File(androidContext().noBackupFilesDir, GRPC_SOCKET_FILE_NAME)
    }
    single {
        ManagementService(
            rpcSocketFile = get(named(GRPC_SOCKET_FILE_NAMED_ARGUMENT)),
            extensiveLogging = BuildConfig.DEBUG,
            scope = MainScope(),
        )
    }

    single { PrepareVpnUseCase(androidContext()) }

    single { BuildVersion(BuildConfig.VERSION_NAME, BuildConfig.VERSION_CODE) }
    single { ApiEndpointFromIntentHolder() }
    single { AccountRepository(get(), get(), MainScope()) }
    single { DeviceRepository(get()) }
    single { ConnectionProxy(get(), get(), get()) }
    single { LocaleRepository(get()) }
    single { RelayLocationTranslationRepository(get(), get(), MainScope()) }
    single { androidContext().resources }
}
