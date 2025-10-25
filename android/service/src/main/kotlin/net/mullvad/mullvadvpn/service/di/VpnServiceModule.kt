package net.mullvad.mullvadvpn.service.di

import android.content.Context
import net.mullvad.mullvadvpn.lib.common.constant.CACHE_DIR_NAMED_ARGUMENT
import net.mullvad.mullvadvpn.lib.common.constant.FILES_DIR_NAMED_ARGUMENT
import net.mullvad.mullvadvpn.lib.common.constant.GRPC_SOCKET_FILE_NAMED_ARGUMENT
import net.mullvad.mullvadvpn.service.DaemonConfig
import net.mullvad.mullvadvpn.service.migration.MigrateSplitTunneling
import org.koin.android.ext.koin.androidContext
import org.koin.core.qualifier.named
import org.koin.dsl.module

val vpnServiceModule = module {
    single(named(FILES_DIR_NAMED_ARGUMENT)) {
        // Use device-protected storage for Direct Boot support.
        val context = androidContext().deviceProtected()
        context.filesDir
    }

    single(named(CACHE_DIR_NAMED_ARGUMENT)) {
        // Use device-protected storage for Direct Boot support.
        val context = androidContext().deviceProtected()
        context.cacheDir
    }

    single { MigrateSplitTunneling(androidContext()) }

    single {
        DaemonConfig(
            rpcSocket = get(named(GRPC_SOCKET_FILE_NAMED_ARGUMENT)),
            filesDir = get(named(FILES_DIR_NAMED_ARGUMENT)),
            cacheDir = get(named(CACHE_DIR_NAMED_ARGUMENT)),
            apiEndpointOverride = getOrNull(),
        )
    }
}

private fun Context.deviceProtected(): Context =
    if (isDeviceProtectedStorage) this else createDeviceProtectedStorageContext()
