package net.mullvad.mullvadvpn.di

import android.content.Context
import androidx.core.app.NotificationManagerCompat
import androidx.datastore.core.DataStore
import androidx.datastore.dataStore
import java.io.File
import kotlinx.coroutines.MainScope
import net.mullvad.mullvadvpn.BuildConfig
import net.mullvad.mullvadvpn.lib.common.constant.GRPC_SOCKET_FILE_NAME
import net.mullvad.mullvadvpn.lib.common.constant.GRPC_SOCKET_FILE_NAMED_ARGUMENT
import net.mullvad.mullvadvpn.lib.daemon.grpc.ManagementService
import net.mullvad.mullvadvpn.lib.endpoint.ApiEndpointFromIntentHolder
import net.mullvad.mullvadvpn.lib.model.BuildVersion
import net.mullvad.mullvadvpn.lib.model.NotificationChannel
import net.mullvad.mullvadvpn.lib.shared.AccountRepository
import net.mullvad.mullvadvpn.lib.shared.ConnectionProxy
import net.mullvad.mullvadvpn.lib.shared.DeviceRepository
import net.mullvad.mullvadvpn.lib.shared.LocaleRepository
import net.mullvad.mullvadvpn.lib.shared.PrepareVpnUseCase
import net.mullvad.mullvadvpn.lib.shared.RelayLocationTranslationRepository
import net.mullvad.mullvadvpn.repository.UserPreferences
import net.mullvad.mullvadvpn.repository.UserPreferencesMigration
import net.mullvad.mullvadvpn.repository.UserPreferencesRepository
import net.mullvad.mullvadvpn.repository.UserPreferencesSerializer
import net.mullvad.mullvadvpn.service.notifications.NotificationChannelFactory
import net.mullvad.mullvadvpn.service.notifications.NotificationManager
import net.mullvad.mullvadvpn.service.notifications.NotificationProvider
import net.mullvad.mullvadvpn.service.notifications.accountexpiry.AccountExpiryScheduledNotificationProvider
import net.mullvad.mullvadvpn.service.notifications.tunnelstate.TunnelStateNotificationProvider
import net.mullvad.mullvadvpn.usecase.ScheduleNotificationAlarmUseCase
import org.koin.android.ext.koin.androidContext
import org.koin.core.module.dsl.createdAtStart
import org.koin.core.module.dsl.withOptions
import org.koin.core.qualifier.named
import org.koin.dsl.bind
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

    single { androidContext().resources }
    single { androidContext().userPreferencesStore }
    single { BuildVersion(BuildConfig.VERSION_NAME, BuildConfig.VERSION_CODE) }
    single { ApiEndpointFromIntentHolder() }
    single { AccountRepository(get(), get(), MainScope()) }
    single { DeviceRepository(get()) }
    single { UserPreferencesRepository(get(), get()) }
    single { ConnectionProxy(get(), get(), get()) }
    single { LocaleRepository(get()) }
    single { RelayLocationTranslationRepository(get(), get(), MainScope()) }
    single { ScheduleNotificationAlarmUseCase(get()) }

    single { NotificationChannel.TunnelUpdates } bind NotificationChannel::class
    single { NotificationChannel.AccountUpdates } bind NotificationChannel::class
    single { NotificationChannelFactory(get(), get(), getAll()) } withOptions { createdAtStart() }
    single { NotificationManagerCompat.from(androidContext()) }
    single { NotificationManager(get(), getAll(), get(), MainScope()) } withOptions
        {
            createdAtStart()
        }
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
        AccountExpiryScheduledNotificationProvider(
            get<NotificationChannel.AccountUpdates>().id,
            get(),
        )
    } bind NotificationProvider::class
}

private val Context.userPreferencesStore: DataStore<UserPreferences> by
    dataStore(
        fileName = APP_PREFERENCES_NAME,
        serializer = UserPreferencesSerializer,
        produceMigrations = UserPreferencesMigration::migrations,
    )
