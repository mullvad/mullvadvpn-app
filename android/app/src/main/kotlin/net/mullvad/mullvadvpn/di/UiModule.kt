package net.mullvad.mullvadvpn.di

import android.content.Context
import android.content.SharedPreferences
import android.content.pm.PackageManager
import android.os.Messenger
import kotlinx.coroutines.Dispatchers
import net.mullvad.mullvadvpn.BuildConfig
import net.mullvad.mullvadvpn.applist.ApplicationsIconManager
import net.mullvad.mullvadvpn.applist.ApplicationsProvider
import net.mullvad.mullvadvpn.lib.ipc.EventDispatcher
import net.mullvad.mullvadvpn.repository.AccountRepository
import net.mullvad.mullvadvpn.repository.ChangelogRepository
import net.mullvad.mullvadvpn.repository.DeviceRepository
import net.mullvad.mullvadvpn.repository.PrivacyDisclaimerRepository
import net.mullvad.mullvadvpn.repository.SettingsRepository
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.SplitTunneling
import net.mullvad.mullvadvpn.util.ChangelogDataProvider
import net.mullvad.mullvadvpn.util.IChangelogDataProvider
import net.mullvad.mullvadvpn.viewmodel.AccountViewModel
import net.mullvad.mullvadvpn.viewmodel.ChangelogViewModel
import net.mullvad.mullvadvpn.viewmodel.ConnectViewModel
import net.mullvad.mullvadvpn.viewmodel.DeviceListViewModel
import net.mullvad.mullvadvpn.viewmodel.DeviceRevokedViewModel
import net.mullvad.mullvadvpn.viewmodel.LoginViewModel
import net.mullvad.mullvadvpn.viewmodel.PrivacyDisclaimerViewModel
import net.mullvad.mullvadvpn.viewmodel.SelectLocationViewModel
import net.mullvad.mullvadvpn.viewmodel.SettingsViewModel
import net.mullvad.mullvadvpn.viewmodel.SplitTunnelingViewModel
import net.mullvad.mullvadvpn.viewmodel.VpnSettingsViewModel
import net.mullvad.mullvadvpn.viewmodel.WelcomeViewModel
import org.apache.commons.validator.routines.InetAddressValidator
import org.koin.android.ext.koin.androidApplication
import org.koin.android.ext.koin.androidContext
import org.koin.androidx.viewmodel.dsl.viewModel
import org.koin.core.qualifier.named
import org.koin.dsl.module
import org.koin.dsl.onClose

val uiModule = module {
    single<SharedPreferences>(named(APP_PREFERENCES_NAME)) {
        androidApplication().getSharedPreferences(APP_PREFERENCES_NAME, Context.MODE_PRIVATE)
    }

    single<PackageManager> { androidContext().packageManager }
    single<String>(named(SELF_PACKAGE_NAME)) { androidContext().packageName }

    viewModel { SplitTunnelingViewModel(get(), get(), Dispatchers.Default) }
    single { ApplicationsIconManager(get()) } onClose { it?.dispose() }
    single { ApplicationsProvider(get(), get(named(SELF_PACKAGE_NAME))) }

    single { (messenger: Messenger, dispatcher: EventDispatcher) ->
        SplitTunneling(messenger, dispatcher)
    }

    single { ServiceConnectionManager(androidContext()) }
    single { InetAddressValidator.getInstance() }
    single { androidContext().resources }
    single { androidContext().assets }

    single { ChangelogRepository(get(named(APP_PREFERENCES_NAME)), get()) }

    single { AccountRepository(get()) }
    single { DeviceRepository(get()) }
    single {
        PrivacyDisclaimerRepository(
            androidContext().getSharedPreferences(APP_PREFERENCES_NAME, Context.MODE_PRIVATE)
        )
    }
    single { SettingsRepository(get()) }

    single<IChangelogDataProvider> { ChangelogDataProvider(get()) }

    // View models
    viewModel { AccountViewModel(get(), get(), get()) }
    viewModel {
        ChangelogViewModel(get(), BuildConfig.VERSION_CODE, BuildConfig.ALWAYS_SHOW_CHANGELOG)
    }
    viewModel { ConnectViewModel(get(), BuildConfig.ENABLE_IN_APP_VERSION_NOTIFICATIONS, get()) }
    viewModel { DeviceListViewModel(get(), get()) }
    viewModel { DeviceRevokedViewModel(get(), get()) }
    viewModel { LoginViewModel(get(), get()) }
    viewModel { PrivacyDisclaimerViewModel(get()) }
    viewModel { SelectLocationViewModel(get()) }
    viewModel { SettingsViewModel(get(), get()) }
    viewModel { VpnSettingsViewModel(get(), get(), get(), get()) }
    viewModel { WelcomeViewModel(get(), get(), get()) }
}

const val SELF_PACKAGE_NAME = "SELF_PACKAGE_NAME"
const val APP_PREFERENCES_NAME = "${BuildConfig.APPLICATION_ID}.app_preferences"
