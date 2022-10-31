package net.mullvad.mullvadvpn.di

import android.content.Context
import android.content.pm.PackageManager
import android.os.Messenger
import kotlinx.coroutines.Dispatchers
import net.mullvad.mullvadvpn.applist.ApplicationsIconManager
import net.mullvad.mullvadvpn.applist.ApplicationsProvider
import net.mullvad.mullvadvpn.ipc.EventDispatcher
import net.mullvad.mullvadvpn.repository.AppChangesRepository
import net.mullvad.mullvadvpn.repository.IChangeLogDataProvider
import net.mullvad.mullvadvpn.ui.notification.AccountExpiryNotification
import net.mullvad.mullvadvpn.ui.notification.TunnelStateNotification
import net.mullvad.mullvadvpn.ui.notification.VersionInfoNotification
import net.mullvad.mullvadvpn.ui.serviceconnection.AccountRepository
import net.mullvad.mullvadvpn.ui.serviceconnection.DeviceRepository
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.SplitTunneling
import net.mullvad.mullvadvpn.util.ChangeLogDataProvider
import net.mullvad.mullvadvpn.viewmodel.AppChangesViewModel
import net.mullvad.mullvadvpn.viewmodel.ConnectViewModel
import net.mullvad.mullvadvpn.viewmodel.DeviceListViewModel
import net.mullvad.mullvadvpn.viewmodel.DeviceRevokedViewModel
import net.mullvad.mullvadvpn.viewmodel.LoginViewModel
import net.mullvad.mullvadvpn.viewmodel.SplitTunnelingViewModel
import org.koin.android.ext.koin.androidApplication
import org.koin.android.ext.koin.androidContext
import org.koin.androidx.viewmodel.dsl.viewModel
import org.koin.core.qualifier.named
import org.koin.dsl.module
import org.koin.dsl.onClose

val uiModule = module {

    single { androidApplication().getSharedPreferences(PREFS_FILE_KEY, Context.MODE_PRIVATE) }

    single<PackageManager> { androidContext().packageManager }
    single<String>(named(SELF_PACKAGE_NAME)) { androidContext().packageName }

    scope(named(APPS_SCOPE)) {
        viewModel { SplitTunnelingViewModel(get(), get(), Dispatchers.Default) }
        scoped { ApplicationsIconManager(get()) } onClose { it?.dispose() }
        scoped { ApplicationsProvider(get(), get(named(SELF_PACKAGE_NAME))) }
    }

    scope(named(SERVICE_CONNECTION_SCOPE)) {
        scoped<SplitTunneling> { (messenger: Messenger, dispatcher: EventDispatcher) ->
            SplitTunneling(messenger, dispatcher)
        }
    }

    single { ServiceConnectionManager(androidContext()) }
    single { androidContext().resources }
    single { androidContext().assets }

    single { AppChangesRepository(get(), get()) }

    single { AccountExpiryNotification(get()) }
    single { TunnelStateNotification(get()) }
    single { VersionInfoNotification(get()) }

    single { AccountRepository(get()) }
    single { DeviceRepository(get()) }

    single<IChangeLogDataProvider> { ChangeLogDataProvider(get()) }

    // View models
    viewModel { ConnectViewModel() }
    viewModel { DeviceRevokedViewModel(get(), get()) }
    viewModel { DeviceListViewModel(get(), get()) }
    viewModel { LoginViewModel(get(), get()) }
    viewModel { AppChangesViewModel(get()) }
}

const val APPS_SCOPE = "APPS_SCOPE"
const val SERVICE_CONNECTION_SCOPE = "SERVICE_CONNECTION_SCOPE"
const val SELF_PACKAGE_NAME = "SELF_PACKAGE_NAME"
const val PREFS_FILE_KEY = "net.mullvad.mullvadvpn.app_preferences"
