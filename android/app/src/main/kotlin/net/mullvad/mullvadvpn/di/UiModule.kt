package net.mullvad.mullvadvpn.di

import android.content.Context
import android.content.SharedPreferences
import android.content.pm.PackageManager
import androidx.datastore.dataStoreFile
import androidx.datastore.preferences.core.PreferenceDataStoreFactory
import java.io.File
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.MainScope
import net.mullvad.mullvadvpn.BuildConfig
import net.mullvad.mullvadvpn.applist.ApplicationsProvider
import net.mullvad.mullvadvpn.constant.IS_PLAY_BUILD
import net.mullvad.mullvadvpn.dataproxy.MullvadProblemReport
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethod
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodId
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodName
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.payment.PaymentProvider
import net.mullvad.mullvadvpn.lib.shared.VoucherRepository
import net.mullvad.mullvadvpn.lib.theme.ThemeRepository
import net.mullvad.mullvadvpn.repository.ApiAccessRepository
import net.mullvad.mullvadvpn.repository.ChangelogRepository
import net.mullvad.mullvadvpn.repository.CustomListsRepository
import net.mullvad.mullvadvpn.repository.InAppNotificationController
import net.mullvad.mullvadvpn.repository.NewDeviceRepository
import net.mullvad.mullvadvpn.repository.PrivacyDisclaimerRepository
import net.mullvad.mullvadvpn.repository.ProblemReportRepository
import net.mullvad.mullvadvpn.repository.RelayListFilterRepository
import net.mullvad.mullvadvpn.repository.RelayListRepository
import net.mullvad.mullvadvpn.repository.RelayOverridesRepository
import net.mullvad.mullvadvpn.repository.SettingsRepository
import net.mullvad.mullvadvpn.repository.SplitTunnelingRepository
import net.mullvad.mullvadvpn.ui.serviceconnection.AppVersionInfoRepository
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.usecase.AccountExpiryNotificationUseCase
import net.mullvad.mullvadvpn.usecase.AvailableProvidersUseCase
import net.mullvad.mullvadvpn.usecase.EmptyPaymentUseCase
import net.mullvad.mullvadvpn.usecase.FilteredRelayListUseCase
import net.mullvad.mullvadvpn.usecase.InternetAvailableUseCase
import net.mullvad.mullvadvpn.usecase.LastKnownLocationUseCase
import net.mullvad.mullvadvpn.usecase.NewDeviceNotificationUseCase
import net.mullvad.mullvadvpn.usecase.OutOfTimeUseCase
import net.mullvad.mullvadvpn.usecase.PaymentUseCase
import net.mullvad.mullvadvpn.usecase.PlayPaymentUseCase
import net.mullvad.mullvadvpn.usecase.SelectedLocationTitleUseCase
import net.mullvad.mullvadvpn.usecase.SystemVpnSettingsAvailableUseCase
import net.mullvad.mullvadvpn.usecase.TunnelStateNotificationUseCase
import net.mullvad.mullvadvpn.usecase.VersionNotificationUseCase
import net.mullvad.mullvadvpn.usecase.customlists.CustomListActionUseCase
import net.mullvad.mullvadvpn.usecase.customlists.CustomListRelayItemsUseCase
import net.mullvad.mullvadvpn.usecase.customlists.CustomListsRelayItemUseCase
import net.mullvad.mullvadvpn.util.ChangelogDataProvider
import net.mullvad.mullvadvpn.util.IChangelogDataProvider
import net.mullvad.mullvadvpn.viewmodel.AccountViewModel
import net.mullvad.mullvadvpn.viewmodel.ApiAccessListViewModel
import net.mullvad.mullvadvpn.viewmodel.ApiAccessMethodDetailsViewModel
import net.mullvad.mullvadvpn.viewmodel.ChangelogViewModel
import net.mullvad.mullvadvpn.viewmodel.ConnectViewModel
import net.mullvad.mullvadvpn.viewmodel.CreateCustomListDialogViewModel
import net.mullvad.mullvadvpn.viewmodel.CustomListLocationsViewModel
import net.mullvad.mullvadvpn.viewmodel.CustomListsViewModel
import net.mullvad.mullvadvpn.viewmodel.DeleteApiAccessMethodConfirmationViewModel
import net.mullvad.mullvadvpn.viewmodel.DeleteCustomListConfirmationViewModel
import net.mullvad.mullvadvpn.viewmodel.DeviceListViewModel
import net.mullvad.mullvadvpn.viewmodel.DeviceRevokedViewModel
import net.mullvad.mullvadvpn.viewmodel.DnsDialogViewModel
import net.mullvad.mullvadvpn.viewmodel.EditApiAccessMethodViewModel
import net.mullvad.mullvadvpn.viewmodel.EditCustomListNameDialogViewModel
import net.mullvad.mullvadvpn.viewmodel.EditCustomListViewModel
import net.mullvad.mullvadvpn.viewmodel.FilterViewModel
import net.mullvad.mullvadvpn.viewmodel.LoginViewModel
import net.mullvad.mullvadvpn.viewmodel.MtuDialogViewModel
import net.mullvad.mullvadvpn.viewmodel.NoDaemonViewModel
import net.mullvad.mullvadvpn.viewmodel.OutOfTimeViewModel
import net.mullvad.mullvadvpn.viewmodel.PaymentViewModel
import net.mullvad.mullvadvpn.viewmodel.PrivacyDisclaimerViewModel
import net.mullvad.mullvadvpn.viewmodel.ReportProblemViewModel
import net.mullvad.mullvadvpn.viewmodel.ResetServerIpOverridesConfirmationViewModel
import net.mullvad.mullvadvpn.viewmodel.SaveApiAccessMethodViewModel
import net.mullvad.mullvadvpn.viewmodel.SelectLocationViewModel
import net.mullvad.mullvadvpn.viewmodel.ServerIpOverridesViewModel
import net.mullvad.mullvadvpn.viewmodel.SettingsViewModel
import net.mullvad.mullvadvpn.viewmodel.SplashViewModel
import net.mullvad.mullvadvpn.viewmodel.SplitTunnelingViewModel
import net.mullvad.mullvadvpn.viewmodel.ViewLogsViewModel
import net.mullvad.mullvadvpn.viewmodel.VoucherDialogViewModel
import net.mullvad.mullvadvpn.viewmodel.VpnPermissionViewModel
import net.mullvad.mullvadvpn.viewmodel.VpnSettingsViewModel
import net.mullvad.mullvadvpn.viewmodel.WelcomeViewModel
import org.apache.commons.validator.routines.InetAddressValidator
import org.koin.android.ext.koin.androidApplication
import org.koin.android.ext.koin.androidContext
import org.koin.androidx.viewmodel.dsl.viewModel
import org.koin.core.qualifier.named
import org.koin.dsl.module

val uiModule = module {
    single<SharedPreferences>(named(APP_PREFERENCES_NAME)) {
        androidApplication().getSharedPreferences(APP_PREFERENCES_NAME, Context.MODE_PRIVATE)
    }

    single<PackageManager> { androidContext().packageManager }
    single<String>(named(SELF_PACKAGE_NAME)) { androidContext().packageName }

    viewModel { SplitTunnelingViewModel(get(), get(), Dispatchers.Default) }
    single { ApplicationsProvider(get(), get(named(SELF_PACKAGE_NAME))) }

    single { ServiceConnectionManager(androidContext()) }
    single { InetAddressValidator.getInstance() }
    single { androidContext().resources }
    single { androidContext().assets }
    single { androidContext().contentResolver }

    single { ChangelogRepository(get(named(APP_PREFERENCES_NAME)), get()) }
    single {
        PrivacyDisclaimerRepository(
            androidContext().getSharedPreferences(APP_PREFERENCES_NAME, Context.MODE_PRIVATE),
        )
    }
    single { SettingsRepository(get()) }
    single { MullvadProblemReport(get()) }
    single { RelayOverridesRepository(get()) }
    single { CustomListsRepository(get()) }
    single { RelayListRepository(get()) }
    single { RelayListFilterRepository(get()) }
    single { VoucherRepository(get(), get()) }
    single { SplitTunnelingRepository(get()) }
    single { ApiAccessRepository(get()) }
    single { NewDeviceRepository() }

    single { AccountExpiryNotificationUseCase(get()) }
    single { TunnelStateNotificationUseCase(get()) }
    single { VersionNotificationUseCase(get(), BuildConfig.ENABLE_IN_APP_VERSION_NOTIFICATIONS) }
    single { NewDeviceNotificationUseCase(get(), get()) }
    single { OutOfTimeUseCase(get(), get(), MainScope()) }
    single { InternetAvailableUseCase(get()) }
    single { SystemVpnSettingsAvailableUseCase(androidContext()) }
    single { CustomListActionUseCase(get(), get()) }
    single { SelectedLocationTitleUseCase(get(), get()) }
    single { AvailableProvidersUseCase(get()) }
    single { CustomListsRelayItemUseCase(get(), get()) }
    single { CustomListRelayItemsUseCase(get(), get()) }
    single { FilteredRelayListUseCase(get(), get()) }
    single { LastKnownLocationUseCase(get()) }

    single { InAppNotificationController(get(), get(), get(), get(), MainScope()) }

    single<IChangelogDataProvider> { ChangelogDataProvider(get()) }

    // Will be resolved using from either of the two PaymentModule.kt classes.
    single { PaymentProvider(get()) }

    single<PaymentUseCase> {
        val paymentRepository = get<PaymentProvider>().paymentRepository
        if (paymentRepository != null) {
            PlayPaymentUseCase(paymentRepository = paymentRepository)
        } else {
            EmptyPaymentUseCase()
        }
    }

    single { ProblemReportRepository() }
    single {
        ThemeRepository(
            PreferenceDataStoreFactory.create {
                File(androidContext().filesDir, "settings.preferences_pb")
            }
        )
    }
    single {
        PreferenceDataStoreFactory.create { androidContext().dataStoreFile(APP_PREFERENCES_NAME) }
    }

    single { AppVersionInfoRepository(get(), get()) }

    // View models
    viewModel { AccountViewModel(get(), get(), get(), IS_PLAY_BUILD) }
    viewModel { ChangelogViewModel(get(), get(), BuildConfig.ALWAYS_SHOW_CHANGELOG) }
    viewModel {
        ConnectViewModel(
            get(),
            get(),
            get(),
            get(),
            get(),
            get(),
            get(),
            get(),
            get(),
            get(),
            IS_PLAY_BUILD
        )
    }
    viewModel { parameters -> DeviceListViewModel(get(), parameters.get()) }
    viewModel { DeviceRevokedViewModel(get(), get()) }
    viewModel { parameters -> MtuDialogViewModel(get(), parameters.getOrNull()) }
    viewModel { parameters ->
        DnsDialogViewModel(get(), get(), parameters.getOrNull(), parameters.getOrNull())
    }
    viewModel { LoginViewModel(get(), get(), get()) }
    viewModel { PrivacyDisclaimerViewModel(get(), IS_PLAY_BUILD) }
    viewModel { SelectLocationViewModel(get(), get(), get(), get(), get(), get()) }
    viewModel { SettingsViewModel(get(), get(), get(), IS_PLAY_BUILD) }
    viewModel { SplashViewModel(get(), get(), get()) }
    viewModel { VoucherDialogViewModel(get()) }
    viewModel { VpnSettingsViewModel(get(), get(), get()) }
    viewModel { WelcomeViewModel(get(), get(), get(), get(), isPlayBuild = IS_PLAY_BUILD) }
    viewModel { ReportProblemViewModel(get(), get()) }
    viewModel { ViewLogsViewModel(get()) }
    viewModel { OutOfTimeViewModel(get(), get(), get(), get(), get(), isPlayBuild = IS_PLAY_BUILD) }
    viewModel { PaymentViewModel(get()) }
    viewModel { FilterViewModel(get(), get()) }
    viewModel { (location: GeoLocationId?) -> CreateCustomListDialogViewModel(location, get()) }
    viewModel { parameters ->
        CustomListLocationsViewModel(parameters.get(), parameters.get(), get(), get(), get())
    }
    viewModel { parameters -> EditCustomListViewModel(parameters.get(), get()) }
    viewModel { parameters ->
        EditCustomListNameDialogViewModel(parameters.get(), parameters.get(), get())
    }
    viewModel { CustomListsViewModel(get(), get()) }
    viewModel { parameters -> DeleteCustomListConfirmationViewModel(parameters.get(), get()) }
    viewModel { ServerIpOverridesViewModel(get(), get()) }
    viewModel { ResetServerIpOverridesConfirmationViewModel(get()) }
    viewModel { VpnPermissionViewModel(get(), get()) }
    viewModel { ApiAccessListViewModel(get()) }
    viewModel { (accessMethodId: ApiAccessMethodId?) ->
        EditApiAccessMethodViewModel(accessMethodId, get(), get())
    }
    viewModel {
        (
            id: ApiAccessMethodId?,
            name: ApiAccessMethodName,
            customProxy: ApiAccessMethod.CustomProxy) ->
        SaveApiAccessMethodViewModel(id, name, customProxy, get())
    }
    viewModel { (accessMethodId: ApiAccessMethodId) ->
        ApiAccessMethodDetailsViewModel(accessMethodId, get())
    }
    viewModel { (accessMethodId: ApiAccessMethodId) ->
        DeleteApiAccessMethodConfirmationViewModel(accessMethodId, get())
    }

    // This view model must be single so we correctly attach lifecycle and share it with activity
    single { NoDaemonViewModel(get()) }
}

const val SELF_PACKAGE_NAME = "SELF_PACKAGE_NAME"
const val APP_PREFERENCES_NAME = "${BuildConfig.APPLICATION_ID}.app_preferences"
