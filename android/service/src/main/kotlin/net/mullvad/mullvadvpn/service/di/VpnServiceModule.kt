package net.mullvad.mullvadvpn.service.di

import net.mullvad.mullvadvpn.lib.common.constant.CACHE_DIR_NAMED_ARGUMENT
import net.mullvad.mullvadvpn.lib.common.constant.FILES_DIR_NAMED_ARGUMENT
import net.mullvad.mullvadvpn.lib.common.constant.GRPC_SOCKET_FILE_NAMED_ARGUMENT
import net.mullvad.mullvadvpn.lib.endpoint.ApiEndpointOverride
import net.mullvad.mullvadvpn.service.BuildConfig
import net.mullvad.mullvadvpn.service.DaemonConfig
import net.mullvad.mullvadvpn.service.migration.MigrateSplitTunneling
import org.koin.android.ext.koin.androidContext
import org.koin.core.qualifier.named
import org.koin.dsl.module

val vpnServiceModule = module {
    single(named(FILES_DIR_NAMED_ARGUMENT)) { androidContext().filesDir }
    single(named(CACHE_DIR_NAMED_ARGUMENT)) { androidContext().cacheDir }

    single { MigrateSplitTunneling(androidContext()) }

    single {
        DaemonConfig(
            rpcSocket = get(named(GRPC_SOCKET_FILE_NAMED_ARGUMENT)),
            filesDir = get(named(FILES_DIR_NAMED_ARGUMENT)),
            cacheDir = get(named(CACHE_DIR_NAMED_ARGUMENT)),
            apiEndpointOverride =
                if (BuildConfig.FLAVOR_infrastructure != "prod") {
                    ApiEndpointOverride(BuildConfig.API_ENDPOINT, BuildConfig.API_IP)
                } else {
                    null
                },
        )
    }
}
