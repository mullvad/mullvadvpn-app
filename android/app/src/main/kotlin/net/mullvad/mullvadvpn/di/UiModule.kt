package net.mullvad.mullvadvpn.di

import android.content.ComponentName
import android.os.Build
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.MainScope
import net.mullvad.mullvadvpn.BuildConfig
import net.mullvad.mullvadvpn.anticensorship.impl.AntiCensorshipSettingsViewModel
import net.mullvad.mullvadvpn.anticensorship.impl.customport.CustomPortDialogViewModel
import net.mullvad.mullvadvpn.anticensorship.impl.selectport.SelectPortViewModel
import net.mullvad.mullvadvpn.apiaccess.impl.screen.delete.DeleteApiAccessMethodConfirmationViewModel
import net.mullvad.mullvadvpn.apiaccess.impl.screen.detail.ApiAccessMethodDetailsViewModel
import net.mullvad.mullvadvpn.apiaccess.impl.screen.edit.EditApiAccessMethodViewModel
import net.mullvad.mullvadvpn.apiaccess.impl.screen.list.ApiAccessListViewModel
import net.mullvad.mullvadvpn.apiaccess.impl.screen.save.SaveApiAccessMethodViewModel
import net.mullvad.mullvadvpn.appearance.impl.AppearanceViewModel
import net.mullvad.mullvadvpn.appinfo.impl.AppInfoViewModel
import net.mullvad.mullvadvpn.appinfo.impl.changelog.ChangelogViewModel
import net.mullvad.mullvadvpn.compose.screen.location.LocationBottomSheetState
import net.mullvad.mullvadvpn.compose.screen.location.RelayListScrollConnection
import net.mullvad.mullvadvpn.customlist.impl.screen.create.CreateCustomListDialogViewModel
import net.mullvad.mullvadvpn.customlist.impl.screen.delete.DeleteCustomListConfirmationViewModel
import net.mullvad.mullvadvpn.customlist.impl.screen.editlist.EditCustomListViewModel
import net.mullvad.mullvadvpn.customlist.impl.screen.editlocations.CustomListLocationsViewModel
import net.mullvad.mullvadvpn.customlist.impl.screen.editname.EditCustomListNameDialogViewModel
import net.mullvad.mullvadvpn.customlist.impl.screen.lists.CustomListsViewModel
import net.mullvad.mullvadvpn.feature.account.impl.AccountViewModel
import net.mullvad.mullvadvpn.feature.addtime.impl.AddTimeViewModel
import net.mullvad.mullvadvpn.feature.autoconnect.impl.AutoConnectAndLockdownModeViewModel
import net.mullvad.mullvadvpn.feature.daita.impl.DaitaViewModel
import net.mullvad.mullvadvpn.feature.home.impl.connect.ConnectViewModel
import net.mullvad.mullvadvpn.feature.home.impl.connect.notificationbanner.InAppNotificationController
import net.mullvad.mullvadvpn.feature.home.impl.devicerevoked.DeviceRevokedViewModel
import net.mullvad.mullvadvpn.feature.home.impl.outoftime.OutOfTimeViewModel
import net.mullvad.mullvadvpn.feature.home.impl.welcome.WelcomeViewModel
import net.mullvad.mullvadvpn.feature.login.impl.LoginViewModel
import net.mullvad.mullvadvpn.feature.login.impl.apiunreachable.ApiUnreachableViewModel
import net.mullvad.mullvadvpn.feature.login.impl.devicelist.DeviceListViewModel
import net.mullvad.mullvadvpn.feature.managedevices.impl.ManageDevicesViewModel
import net.mullvad.mullvadvpn.feature.notification.impl.NotificationSettingsViewModel
import net.mullvad.mullvadvpn.feature.problemreport.impl.ReportProblemViewModel
import net.mullvad.mullvadvpn.feature.problemreport.impl.viewlogs.ViewLogsViewModel
import net.mullvad.mullvadvpn.feature.redeemvoucher.impl.VoucherDialogViewModel
import net.mullvad.mullvadvpn.feature.settings.impl.SettingsViewModel
import net.mullvad.mullvadvpn.feature.splittunneling.impl.SplitTunnelingViewModel
import net.mullvad.mullvadvpn.feature.splittunneling.impl.applist.ApplicationsProvider
import net.mullvad.mullvadvpn.feature.vpnsettings.impl.VpnSettingsViewModel
import net.mullvad.mullvadvpn.feature.vpnsettings.impl.dns.DnsDialogViewModel
import net.mullvad.mullvadvpn.feature.vpnsettings.impl.mtu.MtuDialogViewModel
import net.mullvad.mullvadvpn.filter.impl.FilterViewModel
import net.mullvad.mullvadvpn.lib.common.constant.BillingTypes
import net.mullvad.mullvadvpn.lib.common.constant.BuildTypes
import net.mullvad.mullvadvpn.lib.model.RelayListType
import net.mullvad.mullvadvpn.lib.payment.PaymentProvider
import net.mullvad.mullvadvpn.lib.repository.ApiAccessRepository
import net.mullvad.mullvadvpn.lib.repository.AppVersionInfoRepository
import net.mullvad.mullvadvpn.lib.repository.AutoStartAndConnectOnBootRepository
import net.mullvad.mullvadvpn.lib.repository.ChangelogDataProvider
import net.mullvad.mullvadvpn.lib.repository.ChangelogRepository
import net.mullvad.mullvadvpn.lib.repository.CustomListsRepository
import net.mullvad.mullvadvpn.lib.repository.EmptyPaymentUseCase
import net.mullvad.mullvadvpn.lib.repository.NewDeviceRepository
import net.mullvad.mullvadvpn.lib.repository.PaymentLogic
import net.mullvad.mullvadvpn.lib.repository.PlayPaymentLogic
import net.mullvad.mullvadvpn.lib.repository.ProblemReportRepository
import net.mullvad.mullvadvpn.lib.repository.RelayListFilterRepository
import net.mullvad.mullvadvpn.lib.repository.RelayListRepository
import net.mullvad.mullvadvpn.lib.repository.RelayOverridesRepository
import net.mullvad.mullvadvpn.lib.repository.SettingsRepository
import net.mullvad.mullvadvpn.lib.repository.SplashCompleteRepository
import net.mullvad.mullvadvpn.lib.repository.SplitTunnelingRepository
import net.mullvad.mullvadvpn.lib.repository.VoucherRepository
import net.mullvad.mullvadvpn.lib.repository.WireguardConstraintsRepository
import net.mullvad.mullvadvpn.lib.usecase.DeleteCustomDnsUseCase
import net.mullvad.mullvadvpn.lib.usecase.FilterChipUseCase
import net.mullvad.mullvadvpn.lib.usecase.FilteredRelayListUseCase
import net.mullvad.mullvadvpn.lib.usecase.HopSelectionUseCase
import net.mullvad.mullvadvpn.lib.usecase.InternetAvailableUseCase
import net.mullvad.mullvadvpn.lib.usecase.LastKnownLocationUseCase
import net.mullvad.mullvadvpn.lib.usecase.ModifyAndEnableMultihopUseCase
import net.mullvad.mullvadvpn.lib.usecase.ModifyMultihopUseCase
import net.mullvad.mullvadvpn.lib.usecase.OutOfTimeUseCase
import net.mullvad.mullvadvpn.lib.usecase.ProviderToOwnershipsUseCase
import net.mullvad.mullvadvpn.lib.usecase.RecentsUseCase
import net.mullvad.mullvadvpn.lib.usecase.RelayItemCanBeSelectedUseCase
import net.mullvad.mullvadvpn.lib.usecase.SelectAndEnableMultihopUseCase
import net.mullvad.mullvadvpn.lib.usecase.SelectSinglehopUseCase
import net.mullvad.mullvadvpn.lib.usecase.SelectedLocationTitleUseCase
import net.mullvad.mullvadvpn.lib.usecase.SelectedLocationUseCase
import net.mullvad.mullvadvpn.lib.usecase.SupportEmailUseCase
import net.mullvad.mullvadvpn.lib.usecase.SystemVpnSettingsAvailableUseCase
import net.mullvad.mullvadvpn.lib.usecase.customlists.CustomListActionUseCase
import net.mullvad.mullvadvpn.lib.usecase.customlists.CustomListRelayItemsUseCase
import net.mullvad.mullvadvpn.lib.usecase.customlists.CustomListsRelayItemUseCase
import net.mullvad.mullvadvpn.lib.usecase.customlists.FilterCustomListsRelayItemUseCase
import net.mullvad.mullvadvpn.lib.usecase.inappnotification.AccountExpiryInAppNotificationUseCase
import net.mullvad.mullvadvpn.lib.usecase.inappnotification.Android16UpdateWarningUseCase
import net.mullvad.mullvadvpn.lib.usecase.inappnotification.InAppNotificationUseCase
import net.mullvad.mullvadvpn.lib.usecase.inappnotification.NewChangelogNotificationUseCase
import net.mullvad.mullvadvpn.lib.usecase.inappnotification.NewDeviceNotificationUseCase
import net.mullvad.mullvadvpn.lib.usecase.inappnotification.TunnelStateNotificationUseCase
import net.mullvad.mullvadvpn.lib.usecase.inappnotification.VersionNotificationUseCase
import net.mullvad.mullvadvpn.multihop.impl.MultihopViewModel
import net.mullvad.mullvadvpn.receiver.AutoStartVpnBootCompletedReceiver
import net.mullvad.mullvadvpn.serveripoverride.impl.ServerIpOverridesViewModel
import net.mullvad.mullvadvpn.serveripoverride.impl.reset.ResetServerIpOverridesConfirmationViewModel
import net.mullvad.mullvadvpn.ui.MainActivity
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.util.BackstackObserver
import net.mullvad.mullvadvpn.viewmodel.MullvadAppViewModel
import net.mullvad.mullvadvpn.viewmodel.PrivacyDisclaimerViewModel
import net.mullvad.mullvadvpn.viewmodel.SplashViewModel
import net.mullvad.mullvadvpn.viewmodel.location.LocationBottomSheetViewModel
import net.mullvad.mullvadvpn.viewmodel.location.SearchLocationViewModel
import net.mullvad.mullvadvpn.viewmodel.location.SelectLocationListViewModel
import net.mullvad.mullvadvpn.viewmodel.location.SelectLocationViewModel
import org.apache.commons.validator.routines.InetAddressValidator
import org.koin.android.ext.koin.androidContext
import org.koin.core.module.dsl.viewModel
import org.koin.core.qualifier.named
import org.koin.dsl.bind
import org.koin.dsl.module

val uiModule = module {
    single<ComponentName>(named(BOOT_COMPLETED_RECEIVER_COMPONENT_NAME)) {
        ComponentName(androidContext(), AutoStartVpnBootCompletedReceiver::class.java)
    }

    viewModel { SplitTunnelingViewModel(get(), get(), get(), Dispatchers.Default) }

    single { ApplicationsProvider(get(), get(named(SELF_PACKAGE_NAME))) }
    scope<MainActivity> { scoped { ServiceConnectionManager(androidContext()) } }
    single { InetAddressValidator.getInstance() }
    single { androidContext().assets }
    single { androidContext().contentResolver }

    single { ChangelogRepository(get(), get(), get()) }
    single { SettingsRepository(get()) }
    single {
        ProblemReportRepository(
            context = androidContext(),
            apiEndpointOverride = getOrNull(),
            apiEndpointFromIntentHolder = get(),
            kermitFileLogDirName = KERMIT_FILE_LOG_DIR_NAME,
            accountRepository = get(),
            paymentLogic = get(),
        )
    }
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

    single { AccountExpiryInAppNotificationUseCase(get()) } bind InAppNotificationUseCase::class
    single { TunnelStateNotificationUseCase(get(), get(), get()) } bind
        InAppNotificationUseCase::class
    single {
        VersionNotificationUseCase(get(), BuildConfig.ENABLE_IN_APP_VERSION_NOTIFICATIONS)
    } bind InAppNotificationUseCase::class
    single { NewDeviceNotificationUseCase(get(), get()) } bind InAppNotificationUseCase::class
    single { NewChangelogNotificationUseCase(get()) } bind InAppNotificationUseCase::class
    if (Build.VERSION.SDK_INT == Build.VERSION_CODES.BAKLAVA) {
        single { Android16UpdateWarningUseCase(get(), get()) } bind InAppNotificationUseCase::class
    }

    single { OutOfTimeUseCase(get(), get(), MainScope()) }
    single { InternetAvailableUseCase(get()) }
    single { SystemVpnSettingsAvailableUseCase(androidContext()) }
    single { CustomListActionUseCase(get(), get()) }
    single { SelectedLocationTitleUseCase(get(), get()) }
    single { ProviderToOwnershipsUseCase(get()) }
    single { FilterCustomListsRelayItemUseCase(get(), get(), get()) }
    single { CustomListsRelayItemUseCase(get(), get()) }
    single { CustomListRelayItemsUseCase(get(), get()) }
    single { FilteredRelayListUseCase(get(), get(), get()) }
    single { LastKnownLocationUseCase(get()) }
    single { SelectedLocationUseCase(get(), get()) }
    single { FilterChipUseCase(get(), get(), get()) }
    single { DeleteCustomDnsUseCase(get()) }
    single { RecentsUseCase(get(), get(), get()) }
    single { SelectSinglehopUseCase(relayListRepository = get()) }
    single {
        ModifyMultihopUseCase(
            relayListRepository = get(),
            settingsRepository = get(),
            customListsRepository = get(),
            wireguardConstraintsRepository = get(),
        )
    }
    single {
        SupportEmailUseCase(
            context = androidContext(),
            problemReportRepository = get(),
            buildVersion = get(),
        )
    }
    single {
        HopSelectionUseCase(
            customListRelayItemUseCase = get(),
            relayListRepository = get(),
            settingsRepository = get(),
        )
    }
    single {
        SelectAndEnableMultihopUseCase(relayListRepository = get(), settingsRepository = get())
    }
    single {
        RelayItemCanBeSelectedUseCase(
            filteredRelayListUseCase = get(),
            hopSelectionUseCase = get(),
            settingsRepository = get(),
            relayListRepository = get(),
        )
    }
    single {
        ModifyAndEnableMultihopUseCase(
            relayListRepository = get(),
            settingsRepository = get(),
            customListsRepository = get(),
            wireguardConstraintsRepository = get(),
        )
    }

    single { InAppNotificationController(getAll(), MainScope()) }

    single { ChangelogDataProvider(get()) }

    // Will be resolved using from either of the two PaymentModule.kt classes.
    single { PaymentProvider(get()) }

    single<PaymentLogic> {
        val paymentRepository = get<PaymentProvider>().paymentRepository
        if (paymentRepository != null) {
            PlayPaymentLogic(paymentRepository = paymentRepository)
        } else {
            EmptyPaymentUseCase()
        }
    }

    single { AppVersionInfoRepository(get(), get()) }

    single { RelayListScrollConnection() }

    // View models
    viewModel { AccountViewModel(get(), get(), get()) }
    viewModel { ChangelogViewModel(get(), get(), get()) }
    viewModel {
        AppInfoViewModel(
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
            userPreferencesRepository = get(),
            selectedLocationTitleUseCase = get(),
            outOfTimeUseCase = get(),
            paymentUseCase = get(),
            connectionProxy = get(),
            lastKnownLocationUseCase = get(),
            systemVpnSettingsUseCase = get(),
            resources = get(),
            isPlayBuild = IS_PLAY_BUILD,
            isFdroidBuild = IS_FDROID_BUILD,
            packageName = get(named(SELF_PACKAGE_NAME)),
        )
    }
    viewModel { DeviceListViewModel(get(), get()) }
    viewModel { ManageDevicesViewModel(get(), Dispatchers.IO, get()) }
    viewModel { DeviceRevokedViewModel(get(), get(), get(), get()) }
    viewModel { MtuDialogViewModel(get(), get()) }
    viewModel { DnsDialogViewModel(get(), get(), get(), get()) }
    viewModel { CustomPortDialogViewModel(get()) }
    viewModel { LoginViewModel(get(), get(), get(), get(), get()) }
    viewModel { PrivacyDisclaimerViewModel(get(), IS_PLAY_BUILD) }
    viewModel {
        SelectLocationViewModel(
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
            get(),
        )
    }
    viewModel { SettingsViewModel(get(), get(), get(), get(), IS_PLAY_BUILD) }
    viewModel { SplashViewModel(get(), get(), get(), get()) }
    viewModel { VoucherDialogViewModel(get(), get()) }
    viewModel { VpnSettingsViewModel(get(), get(), get(), get(), get()) }
    viewModel { AntiCensorshipSettingsViewModel(get(), get()) }
    viewModel { WelcomeViewModel(get(), get(), get(), get(), isPlayBuild = IS_PLAY_BUILD) }
    viewModel {
        ReportProblemViewModel(
            mullvadProblemReporter = get(),
            problemReportRepository = get(),
            accountRepository = get(),
            isPlayBuild = IS_PLAY_BUILD,
        )
    }
    viewModel { ViewLogsViewModel(get()) }
    viewModel { OutOfTimeViewModel(get(), get(), get(), get(), get(), isPlayBuild = IS_PLAY_BUILD) }
    viewModel { FilterViewModel(get(), get()) }
    viewModel { CreateCustomListDialogViewModel(get(), get()) }
    viewModel { CustomListLocationsViewModel(get(), get(), get(), get()) }
    viewModel { EditCustomListViewModel(get(), get()) }
    viewModel { EditCustomListNameDialogViewModel(get(), get()) }
    viewModel { CustomListsViewModel(get(), get()) }
    viewModel { DeleteCustomListConfirmationViewModel(get(), get()) }
    viewModel { ServerIpOverridesViewModel(get(), get(), get()) }
    viewModel { ResetServerIpOverridesConfirmationViewModel(get()) }
    viewModel { ApiAccessListViewModel(get()) }
    viewModel { EditApiAccessMethodViewModel(get(), get(), get()) }
    viewModel { SaveApiAccessMethodViewModel(get(), get()) }
    viewModel { ApiAccessMethodDetailsViewModel(get(), get()) }
    viewModel { DeleteApiAccessMethodConfirmationViewModel(get(), get()) }
    viewModel { SelectPortViewModel(get(), get(), get(), get()) }
    viewModel { CustomPortDialogViewModel(get()) }
    viewModel { MultihopViewModel(get(), get()) }
    viewModel { NotificationSettingsViewModel(get()) }
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
            get(),
            get(),
            get(),
        )
    }
    viewModel { (relayListType: RelayListType) ->
        SelectLocationListViewModel(
            relayListType,
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
    viewModel { DaitaViewModel(get(), get()) }
    viewModel {
        AddTimeViewModel(
            paymentUseCase = get(),
            accountRepository = get(),
            connectionProxy = get(),
            isPlayBuild = IS_PLAY_BUILD,
        )
    }
    viewModel {
        ApiUnreachableViewModel(
            apiAccessRepository = get(),
            supportEmailUseCase = get(),
            savedStateHandle = get(),
        )
    }
    viewModel { (locationBottomSheetState: LocationBottomSheetState) ->
        LocationBottomSheetViewModel(
            locationBottomSheetState = locationBottomSheetState,
            canBeSelectedUseCase = get(),
            customListsRelayItemUseCase = get(),
            selectedLocationUseCase = get(),
            modifyMultihopUseCase = get(),
            wireguardConstraintsRepository = get(),
            selectAndEnableMultihopUseCase = get(),
            hopSelectionUseCase = get(),
            modifyAndEnableMultihopUseCase = get(),
        )
    }
    viewModel { AppearanceViewModel(get()) }
    viewModel { AutoConnectAndLockdownModeViewModel(isPlayBuild = IS_PLAY_BUILD) }

    single { BackstackObserver() }

    // This view model must be single so we correctly attach lifecycle and share it with activity
    single { MullvadAppViewModel(get(), get(), get()) }
}

const val APP_PREFERENCES_NAME = "${BuildConfig.APPLICATION_ID}.app_preferences"
const val SELF_PACKAGE_NAME = "SELF_PACKAGE_NAME"
const val KERMIT_FILE_LOG_DIR_NAME = "android_app_logs"

private const val BOOT_COMPLETED_RECEIVER_COMPONENT_NAME = "BOOT_COMPLETED_RECEIVER_COMPONENT_NAME"
private val IS_FDROID_BUILD = BuildConfig.BUILD_TYPE == BuildTypes.FDROID
private val IS_PLAY_BUILD = BuildConfig.FLAVOR_billing == BillingTypes.PLAY
