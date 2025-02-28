package net.mullvad.mullvadvpn.service.di

import androidx.core.app.NotificationManagerCompat
import kotlinx.coroutines.MainScope
import net.mullvad.mullvadvpn.lib.common.constant.CACHE_DIR_NAMED_ARGUMENT
import net.mullvad.mullvadvpn.lib.common.constant.FILES_DIR_NAMED_ARGUMENT
import net.mullvad.mullvadvpn.lib.common.constant.GRPC_SOCKET_FILE_NAMED_ARGUMENT
import net.mullvad.mullvadvpn.lib.endpoint.ApiEndpointOverride
import net.mullvad.mullvadvpn.lib.model.NotificationChannel
import net.mullvad.mullvadvpn.service.BuildConfig
import net.mullvad.mullvadvpn.service.DaemonConfig
import net.mullvad.mullvadvpn.service.migration.MigrateSplitTunneling
import net.mullvad.mullvadvpn.service.notifications.NotificationChannelFactory
import net.mullvad.mullvadvpn.service.notifications.NotificationManager
import net.mullvad.mullvadvpn.service.notifications.NotificationProvider
import net.mullvad.mullvadvpn.service.notifications.accountexpiry.AccountExpiryNotificationProvider
import net.mullvad.mullvadvpn.service.notifications.tunnelstate.TunnelStateNotificationProvider
import net.mullvad.mullvadvpn.widget.MullvadWidgetUpdater
import org.koin.android.ext.koin.androidContext
import org.koin.core.module.dsl.createdAtStart
import org.koin.core.module.dsl.withOptions
import org.koin.core.qualifier.named
import org.koin.dsl.bind
import org.koin.dsl.module

val vpnServiceModule = module {
    single { NotificationManagerCompat.from(androidContext()) }
    single { androidContext().resources }
    single(named(FILES_DIR_NAMED_ARGUMENT)) { androidContext().filesDir }
    single(named(CACHE_DIR_NAMED_ARGUMENT)) { androidContext().cacheDir }

    single { NotificationChannel.TunnelUpdates } bind NotificationChannel::class
    single { NotificationChannel.AccountUpdates } bind NotificationChannel::class
    single { NotificationChannelFactory(get(), get(), getAll()) } withOptions { createdAtStart() }

    single {
        TunnelStateNotificationProvider(
            get(),
            get(),
            get(),
            get<NotificationChannel.TunnelUpdates>().id,
            MainScope(),
        )
    } bind NotificationProvider::class
    single {
        AccountExpiryNotificationProvider(
            get<NotificationChannel.AccountUpdates>().id,
            get(),
            get(),
        )
    } bind NotificationProvider::class

    single { NotificationManager(get(), getAll(), get(), MainScope()) } withOptions
        {
            createdAtStart()
        }

    single { MigrateSplitTunneling(androidContext()) }

    single {
        DaemonConfig(
            rpcSocket = get(named(GRPC_SOCKET_FILE_NAMED_ARGUMENT)),
            filesDir = get(named(FILES_DIR_NAMED_ARGUMENT)),
            cacheDir = get(named(CACHE_DIR_NAMED_ARGUMENT)),
            apiEndpointOverride =
                if (BuildConfig.FLAVOR_infrastructure != "prod") {
                    ApiEndpointOverride(BuildConfig.API_ENDPOINT)
                } else {
                    null
                },
        )
    }
    single {
        MullvadWidgetUpdater(
            androidContext(),
            get(),
            MainScope(),
        )
    }
}
