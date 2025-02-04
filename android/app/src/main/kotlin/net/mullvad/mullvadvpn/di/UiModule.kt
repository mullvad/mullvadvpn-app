package net.mullvad.mullvadvpn.di

import android.content.ComponentName
import android.content.Context
import android.content.pm.PackageManager
import androidx.datastore.core.DataStore
import androidx.datastore.dataStore
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.MainScope
import net.mullvad.mullvadvpn.BuildConfig
import net.mullvad.mullvadvpn.applist.ApplicationsProvider
import net.mullvad.mullvadvpn.compose.state.RelayListType
import net.mullvad.mullvadvpn.constant.IS_FDROID_BUILD
import net.mullvad.mullvadvpn.constant.IS_PLAY_BUILD
import net.mullvad.mullvadvpn.dataproxy.MullvadProblemReport
import net.mullvad.mullvadvpn.lib.payment.PaymentProvider
import net.mullvad.mullvadvpn.lib.shared.VoucherRepository
import net.mullvad.mullvadvpn.receiver.BootCompletedReceiver
import net.mullvad.mullvadvpn.repository.ApiAccessRepository
import net.mullvad.mullvadvpn.repository.AutoStartAndConnectOnBootRepository
import net.mullvad.mullvadvpn.repository.ChangelogRepository
import net.mullvad.mullvadvpn.repository.CustomListsRepository
import net.mullvad.mullvadvpn.repository.InAppNotificationController
import net.mullvad.mullvadvpn.repository.NewDeviceRepository
import net.mullvad.mullvadvpn.repository.ProblemReportRepository
import net.mullvad.mullvadvpn.repository.RelayListFilterRepository
import net.mullvad.mullvadvpn.repository.RelayListRepository
import net.mullvad.mullvadvpn.repository.RelayOverridesRepository
import net.mullvad.mullvadvpn.repository.SettingsRepository
import net.mullvad.mullvadvpn.repository.SplashCompleteRepository
import net.mullvad.mullvadvpn.repository.SplitTunnelingRepository
import net.mullvad.mullvadvpn.repository.UserPreferences
import net.mullvad.mullvadvpn.repository.UserPreferencesMigration
import net.mullvad.mullvadvpn.repository.UserPreferencesRepository
import net.mullvad.mullvadvpn.repository.UserPreferencesSerializer
import net.mullvad.mullvadvpn.repository.WireguardConstraintsRepository
import net.mullvad.mullvadvpn.service.DaemonConfig
import net.mullvad.mullvadvpn.ui.MainActivity
import net.mullvad.mullvadvpn.ui.serviceconnection.AppVersionInfoRepository
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.usecase.AccountExpiryInAppNotificationUseCase
import net.mullvad.mullvadvpn.usecase.DeleteCustomDnsUseCase
import net.mullvad.mullvadvpn.usecase.EmptyPaymentUseCase
import net.mullvad.mullvadvpn.usecase.FilterChipUseCase
import net.mullvad.mullvadvpn.usecase.FilteredRelayListUseCase
import net.mullvad.mullvadvpn.usecase.InternetAvailableUseCase
import net.mullvad.mullvadvpn.usecase.LastKnownLocationUseCase
import net.mullvad.mullvadvpn.usecase.NewChangelogNotificationUseCase
import net.mullvad.mullvadvpn.usecase.NewDeviceNotificationUseCase
import net.mullvad.mullvadvpn.usecase.OutOfTimeUseCase
import net.mullvad.mullvadvpn.usecase.PaymentUseCase
import net.mullvad.mullvadvpn.usecase.PlayPaymentUseCase
import net.mullvad.mullvadvpn.usecase.ProviderToOwnershipsUseCase
import net.mullvad.mullvadvpn.usecase.SelectedLocationTitleUseCase
import net.mullvad.mullvadvpn.usecase.SelectedLocationUseCase
import net.mullvad.mullvadvpn.usecase.SystemVpnSettingsAvailableUseCase
import net.mullvad.mullvadvpn.usecase.TunnelStateNotificationUseCase
import net.mullvad.mullvadvpn.usecase.VersionNotificationUseCase
import net.mullvad.mullvadvpn.usecase.customlists.CustomListActionUseCase
import net.mullvad.mullvadvpn.usecase.customlists.CustomListRelayItemsUseCase
import net.mullvad.mullvadvpn.usecase.customlists.CustomListsRelayItemUseCase
import net.mullvad.mullvadvpn.usecase.customlists.FilterCustomListsRelayItemUseCase
import net.mullvad.mullvadvpn.util.ChangelogDataProvider
import net.mullvad.mullvadvpn.util.IChangelogDataProvider
import net.mullvad.mullvadvpn.viewmodel.AccountViewModel
import net.mullvad.mullvadvpn.viewmodel.ApiAccessListViewModel
import net.mullvad.mullvadvpn.viewmodel.ApiAccessMethodDetailsViewModel
import net.mullvad.mullvadvpn.viewmodel.AppInfoViewModel
import net.mullvad.mullvadvpn.viewmodel.ChangelogViewModel
import net.mullvad.mullvadvpn.viewmodel.ConnectViewModel
import net.mullvad.mullvadvpn.viewmodel.CreateCustomListDialogViewModel
import net.mullvad.mullvadvpn.viewmodel.CustomListLocationsViewModel
import net.mullvad.mullvadvpn.viewmodel.CustomListsViewModel
import net.mullvad.mullvadvpn.viewmodel.DaitaViewModel
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
import net.mullvad.mullvadvpn.viewmodel.MullvadAppViewModel
import net.mullvad.mullvadvpn.viewmodel.MultihopViewModel
import net.mullvad.mullvadvpn.viewmodel.OutOfTimeViewModel
import net.mullvad.mullvadvpn.viewmodel.PaymentViewModel
import net.mullvad.mullvadvpn.viewmodel.PrivacyDisclaimerViewModel
import net.mullvad.mullvadvpn.viewmodel.ReportProblemViewModel
import net.mullvad.mullvadvpn.viewmodel.ResetServerIpOverridesConfirmationViewModel
import net.mullvad.mullvadvpn.viewmodel.SaveApiAccessMethodViewModel
import net.mullvad.mullvadvpn.viewmodel.ServerIpOverridesViewModel
import net.mullvad.mullvadvpn.viewmodel.SettingsViewModel
import net.mullvad.mullvadvpn.viewmodel.ShadowsocksCustomPortDialogViewModel
import net.mullvad.mullvadvpn.viewmodel.ShadowsocksSettingsViewModel
import net.mullvad.mullvadvpn.viewmodel.SplashViewModel
import net.mullvad.mullvadvpn.viewmodel.SplitTunnelingViewModel
import net.mullvad.mullvadvpn.viewmodel.Udp2TcpSettingsViewModel
import net.mullvad.mullvadvpn.viewmodel.ViewLogsViewModel
import net.mullvad.mullvadvpn.viewmodel.VoucherDialogViewModel
import net.mullvad.mullvadvpn.viewmodel.VpnSettingsViewModel
import net.mullvad.mullvadvpn.viewmodel.WelcomeViewModel
import net.mullvad.mullvadvpn.viewmodel.WireguardCustomPortDialogViewModel
import net.mullvad.mullvadvpn.viewmodel.location.SearchLocationViewModel
import net.mullvad.mullvadvpn.viewmodel.location.SelectLocationListViewModel
import net.mullvad.mullvadvpn.viewmodel.location.SelectLocationViewModel
import org.apache.commons.validator.routines.InetAddressValidator
import org.koin.android.ext.koin.androidContext
import org.koin.core.module.dsl.viewModel
import org.koin.core.qualifier.named
import org.koin.dsl.module

val uiModule = module {
    single<DataStore<UserPreferences>>(named("COOL")) { androidContext().userPreferencesStore }

    single<PackageManager> { androidContext().packageManager }
    single<String>(named(SELF_PACKAGE_NAME)) { androidContext().packageName }

    single<ComponentName>(named(BOOT_COMPLETED_RECEIVER_COMPONENT_NAME)) {
        ComponentName(androidContext(), BootCompletedReceiver::class.java)
    }

    viewModel { SplitTunnelingViewModel(get(), get(), Dispatchers.Default) }

    single { ApplicationsProvider(get(), get(named(SELF_PACKAGE_NAME))) }
    scope<MainActivity> { scoped { ServiceConnectionManager(androidContext()) } }
    single { InetAddressValidator.getInstance() }
    single { androidContext().assets }
    single { androidContext().contentResolver }

    single { ChangelogRepository(get(), get(), get()) }
    single<UserPreferencesRepository> {
        UserPreferencesRepository(get<DataStore<UserPreferences>>(named("COOL")), get())
    }
    single { SettingsRepository(get()) }
    single { MullvadProblemReport(get(), get<DaemonConfig>().apiEndpointOverride, get()) }
    single { RelayOverridesRepository(get()) }
    single { CustomListsRepository(get()) }
    single { RelayListRepository(get(), get()) }
    single { RelayListFilterRepository(get()) }
    single { VoucherRepository(get(), get()) }
    single { SplitTunnelingRepository(get()) }
    single { ApiAccessRepository(get()) }
    single { NewDeviceRepository() }
    single { SplashCompleteRepository() }
    single {
        AutoStartAndConnectOnBootRepository(
            get(),
            get(named(BOOT_COMPLETED_RECEIVER_COMPONENT_NAME)),
        )
    }
    single { WireguardConstraintsRepository(get()) }

    single { AccountExpiryInAppNotificationUseCase(get()) }
    single { TunnelStateNotificationUseCase(get()) }
    single { VersionNotificationUseCase(get(), BuildConfig.ENABLE_IN_APP_VERSION_NOTIFICATIONS) }
    single { NewDeviceNotificationUseCase(get(), get()) }
    single { NewChangelogNotificationUseCase(get()) }
    single { OutOfTimeUseCase(get(), get(), MainScope()) }
    single { InternetAvailableUseCase(get()) }
    single { SystemVpnSettingsAvailableUseCase(androidContext()) }
    single { CustomListActionUseCase(get(), get()) }
    single { SelectedLocationTitleUseCase(get(), get()) }
    single { ProviderToOwnershipsUseCase(get()) }
    single { FilterCustomListsRelayItemUseCase(get(), get(), get(), get()) }
    single { CustomListsRelayItemUseCase(get(), get()) }
    single { CustomListRelayItemsUseCase(get(), get()) }
    single { FilteredRelayListUseCase(get(), get(), get(), get()) }
    single { LastKnownLocationUseCase(get()) }
    single { SelectedLocationUseCase(get(), get()) }
    single { FilterChipUseCase(get(), get(), get(), get()) }
    single { DeleteCustomDnsUseCase(get()) }

    single { InAppNotificationController(get(), get(), get(), get(), get(), MainScope()) }

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

    single { AppVersionInfoRepository(get(), get()) }

    // View models
    viewModel { AccountViewModel(get(), get(), get(), IS_PLAY_BUILD) }
    viewModel { ChangelogViewModel(get(), get(), get()) }
    viewModel {
        AppInfoViewModel(
            changelogRepository = get(),
            appVersionInfoRepository = get(),
            resources = get(),
            isPlayBuild = IS_PLAY_BUILD,
            isFdroidBuild = IS_FDROID_BUILD,
            packageName = get(named(SELF_PACKAGE_NAME)),
        )
    }
    viewModel {
        ConnectViewModel(
            accountRepository = get(),
            deviceRepository = get(),
            changelogRepository = get(),
            inAppNotificationController = get(),
            newDeviceRepository = get(),
            selectedLocationTitleUseCase = get(),
            outOfTimeUseCase = get(),
            paymentUseCase = get(),
            connectionProxy = get(),
            lastKnownLocationUseCase = get(),
            resources = get(),
            isPlayBuild = IS_PLAY_BUILD,
            isFdroidBuild = IS_FDROID_BUILD,
            packageName = get(named(SELF_PACKAGE_NAME)),
        )
    }
    viewModel { DeviceListViewModel(get(), get()) }
    viewModel { DeviceRevokedViewModel(get(), get()) }
    viewModel { MtuDialogViewModel(get(), get()) }
    viewModel { DnsDialogViewModel(get(), get(), get(), get()) }
    viewModel { WireguardCustomPortDialogViewModel(get()) }
    viewModel { LoginViewModel(get(), get(), get()) }
    viewModel { PrivacyDisclaimerViewModel(get(), IS_PLAY_BUILD) }
    viewModel { SelectLocationViewModel(get(), get(), get(), get(), get(), get(), get()) }
    viewModel { SettingsViewModel(get(), get(), get(), get(), IS_PLAY_BUILD) }
    viewModel { SplashViewModel(get(), get(), get(), get()) }
    viewModel { VoucherDialogViewModel(get()) }
    viewModel { VpnSettingsViewModel(get(), get(), get(), get(), get(), get(), get()) }
    viewModel { WelcomeViewModel(get(), get(), get(), get(), isPlayBuild = IS_PLAY_BUILD) }
    viewModel { ReportProblemViewModel(get(), get()) }
    viewModel { ViewLogsViewModel(get()) }
    viewModel { OutOfTimeViewModel(get(), get(), get(), get(), get(), isPlayBuild = IS_PLAY_BUILD) }
    viewModel { PaymentViewModel(get()) }
    viewModel { FilterViewModel(get(), get()) }
    viewModel { CreateCustomListDialogViewModel(get(), get()) }
    viewModel { CustomListLocationsViewModel(get(), get(), get(), get()) }
    viewModel { EditCustomListViewModel(get(), get()) }
    viewModel { EditCustomListNameDialogViewModel(get(), get()) }
    viewModel { CustomListsViewModel(get(), get()) }
    viewModel { DeleteCustomListConfirmationViewModel(get(), get()) }
    viewModel { ServerIpOverridesViewModel(get(), get()) }
    viewModel { ResetServerIpOverridesConfirmationViewModel(get()) }
    viewModel { ApiAccessListViewModel(get()) }
    viewModel { EditApiAccessMethodViewModel(get(), get(), get()) }
    viewModel { SaveApiAccessMethodViewModel(get(), get()) }
    viewModel { ApiAccessMethodDetailsViewModel(get(), get()) }
    viewModel { DeleteApiAccessMethodConfirmationViewModel(get(), get()) }
    viewModel { Udp2TcpSettingsViewModel(get()) }
    viewModel { ShadowsocksSettingsViewModel(get(), get()) }
    viewModel { ShadowsocksCustomPortDialogViewModel(get()) }
    viewModel { MultihopViewModel(get()) }
    viewModel {
        SearchLocationViewModel(
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
            get(),
        )
    }
    viewModel { (relayListType: RelayListType) ->
        SelectLocationListViewModel(relayListType, get(), get(), get(), get(), get(), get(), get())
    }
    viewModel { DaitaViewModel(get()) }

    // This view model must be single so we correctly attach lifecycle and share it with activity
    single { MullvadAppViewModel(get(), get()) }
}

const val SELF_PACKAGE_NAME = "SELF_PACKAGE_NAME"
const val APP_PREFERENCES_NAME = "${BuildConfig.APPLICATION_ID}.app_preferences"
const val BOOT_COMPLETED_RECEIVER_COMPONENT_NAME = "BOOT_COMPLETED_RECEIVER_COMPONENT_NAME"

private val Context.userPreferencesStore: DataStore<UserPreferences> by
    dataStore(
        fileName = APP_PREFERENCES_NAME,
        serializer = UserPreferencesSerializer,
        produceMigrations = UserPreferencesMigration::migrations,
    )
