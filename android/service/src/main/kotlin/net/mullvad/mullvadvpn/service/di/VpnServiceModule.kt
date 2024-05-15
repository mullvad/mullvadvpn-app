package net.mullvad.mullvadvpn.service.di

import androidx.core.app.NotificationManagerCompat
import kotlinx.coroutines.MainScope
import net.mullvad.mullvadvpn.model.NotificationChannel
import net.mullvad.mullvadvpn.service.MigrateSplitTunneling
import net.mullvad.mullvadvpn.service.notifications.ChannelFactory
import net.mullvad.mullvadvpn.service.notifications.NotificationManager
import net.mullvad.mullvadvpn.service.notifications.NotificationProvider
import net.mullvad.mullvadvpn.service.notifications.accountexpiry.AccountExpiryNotificationProvider
import net.mullvad.mullvadvpn.service.notifications.tunnelstate.TunnelStateNotificationProvider
import org.koin.android.ext.koin.androidContext
import org.koin.core.module.dsl.createdAtStart
import org.koin.core.module.dsl.withOptions
import org.koin.dsl.bind
import org.koin.dsl.module

val vpnServiceModule = module {
    single { NotificationManagerCompat.from(androidContext()) }
    single { androidContext().resources }

    single { NotificationChannel.TunnelUpdates } bind NotificationChannel::class
    single { NotificationChannel.AccountUpdates } bind NotificationChannel::class
    single { ChannelFactory(get(), get(), getAll()) } withOptions { createdAtStart() }

    single {
        TunnelStateNotificationProvider(
            get(),
            get(),
            get(),
            get<NotificationChannel.TunnelUpdates>().id,
            MainScope()
        )
    } bind NotificationProvider::class
    single {
        AccountExpiryNotificationProvider(get<NotificationChannel.AccountUpdates>().id, get())
    } bind NotificationProvider::class

    single {
        NotificationManager(
            get(),
            getAll(),
            get(),
            MainScope(),
        )
    } withOptions { createdAtStart() }

    single { MigrateSplitTunneling(androidContext()) }
}
