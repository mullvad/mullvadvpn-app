package net.mullvad.mullvadvpn.service.di

import androidx.core.app.NotificationManagerCompat
import org.koin.android.ext.koin.androidContext
import org.koin.dsl.module

val vpnServiceModule = module { single { NotificationManagerCompat.from(androidContext()) } }
