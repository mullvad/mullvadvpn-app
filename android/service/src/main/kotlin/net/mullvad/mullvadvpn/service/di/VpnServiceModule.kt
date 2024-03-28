package net.mullvad.mullvadvpn.service.di

import androidx.core.app.NotificationManagerCompat
import kotlinx.coroutines.MainScope
import net.mullvad.mullvadvpn.lib.daemon.grpc.ManagementService
import org.koin.android.ext.koin.androidContext
import org.koin.dsl.module

val vpnServiceModule = module {
    single { NotificationManagerCompat.from(androidContext()) }
    single { ManagementService("/data/data/net.mullvad.mullvadvpn/rpc-socket", MainScope()) }
}
