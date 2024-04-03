package net.mullvad.mullvadvpn.di

import android.annotation.SuppressLint
import android.content.Context
import android.content.SharedPreferences
import android.content.pm.PackageManager
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.MainScope
import net.mullvad.mullvadvpn.BuildConfig
import net.mullvad.mullvadvpn.applist.ApplicationsProvider
import net.mullvad.mullvadvpn.constant.IS_PLAY_BUILD
import net.mullvad.mullvadvpn.dataproxy.MullvadProblemReport
import net.mullvad.mullvadvpn.lib.daemon.grpc.ManagementService
import net.mullvad.mullvadvpn.lib.payment.PaymentProvider
import net.mullvad.mullvadvpn.model.GeographicLocationConstraint
import net.mullvad.mullvadvpn.repository.AccountRepository
import net.mullvad.mullvadvpn.repository.ChangelogRepository
import net.mullvad.mullvadvpn.repository.CustomListsRepository
import net.mullvad.mullvadvpn.repository.DeviceRepository
import net.mullvad.mullvadvpn.repository.InAppNotificationController
import net.mullvad.mullvadvpn.repository.PrivacyDisclaimerRepository
import net.mullvad.mullvadvpn.repository.ProblemReportRepository
import net.mullvad.mullvadvpn.repository.RelayOverridesRepository
import net.mullvad.mullvadvpn.repository.SettingsRepository
import net.mullvad.mullvadvpn.ui.serviceconnection.AppVersionInfoCache
import net.mullvad.mullvadvpn.ui.serviceconnection.ConnectionProxy
import net.mullvad.mullvadvpn.ui.serviceconnection.RelayListListener
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.usecase.AccountExpiryNotificationUseCase
import net.mullvad.mullvadvpn.usecase.ConnectivityUseCase
import net.mullvad.mullvadvpn.usecase.EmptyPaymentUseCase
import net.mullvad.mullvadvpn.usecase.NewDeviceNotificationUseCase
import net.mullvad.mullvadvpn.usecase.OutOfTimeUseCase
import net.mullvad.mullvadvpn.usecase.PaymentUseCase
import net.mullvad.mullvadvpn.usecase.PlayPaymentUseCase
import net.mullvad.mullvadvpn.usecase.PortRangeUseCase
import net.mullvad.mullvadvpn.usecase.RelayListFilterUseCase
import net.mullvad.mullvadvpn.usecase.RelayListUseCase
import net.mullvad.mullvadvpn.usecase.SystemVpnSettingsUseCase
import net.mullvad.mullvadvpn.usecase.TunnelStateNotificationUseCase
import net.mullvad.mullvadvpn.usecase.VersionNotificationUseCase
import net.mullvad.mullvadvpn.usecase.customlists.CustomListActionUseCase
import net.mullvad.mullvadvpn.util.ChangelogDataProvider
import net.mullvad.mullvadvpn.util.IChangelogDataProvider
import net.mullvad.mullvadvpn.viewmodel.AccountViewModel
import net.mullvad.mullvadvpn.viewmodel.ChangelogViewModel
import net.mullvad.mullvadvpn.viewmodel.ConnectViewModel
import net.mullvad.mullvadvpn.viewmodel.CreateCustomListDialogViewModel
import net.mullvad.mullvadvpn.viewmodel.CustomListLocationsViewModel
import net.mullvad.mullvadvpn.viewmodel.CustomListsViewModel
import net.mullvad.mullvadvpn.viewmodel.DeleteCustomListConfirmationViewModel
import net.mullvad.mullvadvpn.viewmodel.DeviceListViewModel
import net.mullvad.mullvadvpn.viewmodel.DeviceRevokedViewModel
import net.mullvad.mullvadvpn.viewmodel.DnsDialogViewModel
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
import net.mullvad.mullvadvpn.viewmodel.SelectLocationViewModel
import net.mullvad.mullvadvpn.viewmodel.ServerIpOverridesViewModel
import net.mullvad.mullvadvpn.viewmodel.SettingsViewModel
import net.mullvad.mullvadvpn.viewmodel.SplashViewModel
import net.mullvad.mullvadvpn.viewmodel.SplitTunnelingViewModel
import net.mullvad.mullvadvpn.viewmodel.ViewLogsViewModel
import net.mullvad.mullvadvpn.viewmodel.VoucherDialogViewModel
import net.mullvad.mullvadvpn.viewmodel.VpnSettingsViewModel
import net.mullvad.mullvadvpn.viewmodel.WelcomeViewModel
import org.apache.commons.validator.routines.InetAddressValidator
import org.koin.android.ext.koin.androidApplication
import org.koin.android.ext.koin.androidContext
import org.koin.androidx.viewmodel.dsl.viewModel
import org.koin.core.qualifier.named
import org.koin.dsl.module

@SuppressLint("SdCardPath")
val uiModule = module {
    single<SharedPreferences>(named(APP_PREFERENCES_NAME)) {
        androidApplication().getSharedPreferences(APP_PREFERENCES_NAME, Context.MODE_PRIVATE)
    }

    single<PackageManager> { androidContext().packageManager }
    single<String>(named(SELF_PACKAGE_NAME)) { androidContext().packageName }

    viewModel { SplitTunnelingViewModel(get(), Dispatchers.Default) }
    single { ApplicationsProvider(get(), get(named(SELF_PACKAGE_NAME))) }

    single { ServiceConnectionManager(androidContext()) }
    single { InetAddressValidator.getInstance() }
    single { androidContext().resources }
    single { androidContext().assets }
    single { androidContext().contentResolver }

    single { ChangelogRepository(get(named(APP_PREFERENCES_NAME)), get()) }

    single { AccountRepository(get(), MainScope()) }
    single { DeviceRepository(get()) }
    single {
        PrivacyDisclaimerRepository(
            androidContext().getSharedPreferences(APP_PREFERENCES_NAME, Context.MODE_PRIVATE)
        )
    }
    single { SettingsRepository(get(), get()) }
    single { MullvadProblemReport(get()) }
    single { RelayOverridesRepository(get(), get()) }
    single { CustomListsRepository(get(), get(), get()) }
    single { CustomListsRepository(get(), get()) }

    single { AccountExpiryNotificationUseCase(get()) }
    single { TunnelStateNotificationUseCase(get()) }
    single { VersionNotificationUseCase(get(), BuildConfig.ENABLE_IN_APP_VERSION_NOTIFICATIONS) }
    single { NewDeviceNotificationUseCase(get()) }
    single { PortRangeUseCase(get()) }
    single { RelayListUseCase(get(), get()) }
    single { OutOfTimeUseCase(get(), MainScope()) }
    single { ConnectivityUseCase(get()) }
    single { SystemVpnSettingsUseCase(androidContext()) }
    single { CustomListActionUseCase(get(), get()) }

    single { InAppNotificationController(get(), get(), get(), get(), MainScope()) }
    single { ManagementService("/data/data/net.mullvad.mullvadvpn/rpc-socket", MainScope()) }

    single<IChangelogDataProvider> { ChangelogDataProvider(get()) }

    single { RelayListFilterUseCase(get(), get()) }
    single { RelayListListener(get()) }

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

    single { ConnectionProxy(get()) }
    single { AppVersionInfoCache(get()) }

    // View models
    viewModel { AccountViewModel(get(), get(), get(), IS_PLAY_BUILD) }
    viewModel {
        ChangelogViewModel(get(), BuildConfig.VERSION_CODE, BuildConfig.ALWAYS_SHOW_CHANGELOG)
    }
    viewModel {
        ConnectViewModel(get(), get(), get(), get(), get(), get(), get(), get(), IS_PLAY_BUILD)
    }
    viewModel { DeviceListViewModel(get(), get()) }
    viewModel { DeviceRevokedViewModel(get(), get(), get()) }
    viewModel { MtuDialogViewModel(get()) }
    viewModel { parameters ->
        DnsDialogViewModel(get(), get(), parameters.getOrNull(), parameters.getOrNull())
    }
    viewModel { LoginViewModel(get(), get(), get(), get()) }
    viewModel { PrivacyDisclaimerViewModel(get(), IS_PLAY_BUILD) }
    viewModel { SelectLocationViewModel(get(), get(), get(), get()) }
    viewModel { SettingsViewModel(get(), get(), IS_PLAY_BUILD) }
    viewModel { SplashViewModel(get(), get()) }
    viewModel { VoucherDialogViewModel(get()) }
    viewModel { VpnSettingsViewModel(get(), get(), get(), get(), get()) }
    viewModel { WelcomeViewModel(get(), get(), get(), get(), get(), isPlayBuild = IS_PLAY_BUILD) }
    viewModel { ReportProblemViewModel(get(), get()) }
    viewModel { ViewLogsViewModel(get()) }
    viewModel {
        OutOfTimeViewModel(get(), get(), get(), get(), get(), get(), isPlayBuild = IS_PLAY_BUILD)
    }
    viewModel { PaymentViewModel(get()) }
    viewModel { FilterViewModel(get()) }
    viewModel { (location: GeographicLocationConstraint?) ->
        CreateCustomListDialogViewModel(location, get())
    }
    viewModel { parameters ->
        CustomListLocationsViewModel(parameters.get(), parameters.get(), get(), get())
    }
    viewModel { parameters -> EditCustomListViewModel(parameters.get(), get()) }
    viewModel { parameters ->
        EditCustomListNameDialogViewModel(parameters.get(), parameters.get(), get())
    }
    viewModel { CustomListsViewModel(get(), get()) }
    viewModel { parameters -> DeleteCustomListConfirmationViewModel(parameters.get(), get()) }
    viewModel { ServerIpOverridesViewModel(get(), get(), get(), get()) }
    viewModel { ResetServerIpOverridesConfirmationViewModel(get()) }

    // This view model must be single so we correctly attach lifecycle and share it with activity
    single { NoDaemonViewModel(get()) }
}

const val SELF_PACKAGE_NAME = "SELF_PACKAGE_NAME"
const val APP_PREFERENCES_NAME = "${BuildConfig.APPLICATION_ID}.app_preferences"
