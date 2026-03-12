@file:Suppress("MatchingDeclarationName")

package net.mullvad.mullvadvpn.app

import android.Manifest
import android.os.Build
import androidx.annotation.RequiresApi
import androidx.compose.animation.ExperimentalSharedTransitionApi
import androidx.compose.animation.SharedTransitionLayout
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.semantics.testTagsAsResourceId
import androidx.navigation.NavHostController
import androidx.navigation3.runtime.entryProvider
import androidx.navigation3.scene.DialogSceneStrategy
import androidx.navigation3.scene.SinglePaneSceneStrategy
import androidx.navigation3.ui.NavDisplay
import co.touchlab.kermit.Logger
import com.google.accompanist.permissions.ExperimentalPermissionsApi
import com.google.accompanist.permissions.isGranted
import com.google.accompanist.permissions.rememberPermissionState
import com.ramcosta.composedestinations.annotation.ExternalDestination
import com.ramcosta.composedestinations.annotation.NavHostGraph
import com.ramcosta.composedestinations.generated.account.destinations.AccountDestination
import com.ramcosta.composedestinations.generated.addtime.destinations.VerificationPendingDestination
import com.ramcosta.composedestinations.generated.anticensorship.destinations.AntiCensorshipSettingsDestination
import com.ramcosta.composedestinations.generated.anticensorship.destinations.CustomPortDestination
import com.ramcosta.composedestinations.generated.anticensorship.destinations.SelectPortDestination
import com.ramcosta.composedestinations.generated.apiaccess.destinations.ApiAccessListDestination
import com.ramcosta.composedestinations.generated.apiaccess.destinations.ApiAccessMethodDetailsDestination
import com.ramcosta.composedestinations.generated.apiaccess.destinations.ApiAccessMethodInfoDestination
import com.ramcosta.composedestinations.generated.apiaccess.destinations.DeleteApiAccessMethodConfirmationDestination
import com.ramcosta.composedestinations.generated.apiaccess.destinations.DiscardApiAccessChangesDestination
import com.ramcosta.composedestinations.generated.apiaccess.destinations.EditApiAccessMethodDestination
import com.ramcosta.composedestinations.generated.apiaccess.destinations.EncryptedDnsProxyInfoDestination
import com.ramcosta.composedestinations.generated.apiaccess.destinations.SaveApiAccessMethodDestination
import com.ramcosta.composedestinations.generated.appearance.destinations.AppearanceDestination
import com.ramcosta.composedestinations.generated.appinfo.destinations.AppInfoDestination
import com.ramcosta.composedestinations.generated.appinfo.destinations.ChangelogDestination
import com.ramcosta.composedestinations.generated.autoconnect.destinations.AutoConnectAndLockdownModeDestination
import com.ramcosta.composedestinations.generated.customlist.destinations.CreateCustomListDestination
import com.ramcosta.composedestinations.generated.customlist.destinations.CustomListLocationsDestination
import com.ramcosta.composedestinations.generated.customlist.destinations.CustomListsDestination
import com.ramcosta.composedestinations.generated.customlist.destinations.DeleteCustomListDestination
import com.ramcosta.composedestinations.generated.customlist.destinations.DiscardChangesDestination
import com.ramcosta.composedestinations.generated.customlist.destinations.EditCustomListDestination
import com.ramcosta.composedestinations.generated.customlist.destinations.EditCustomListNameDestination
import com.ramcosta.composedestinations.generated.daita.destinations.DaitaDestination
import com.ramcosta.composedestinations.generated.daita.destinations.DaitaDirectOnlyConfirmationDestination
import com.ramcosta.composedestinations.generated.daita.destinations.DaitaDirectOnlyInfoDestination
import com.ramcosta.composedestinations.generated.deleteaccount.destinations.DeleteAccountCompleteDestination
import com.ramcosta.composedestinations.generated.deleteaccount.destinations.DeleteAccountConfirmationDestination
import com.ramcosta.composedestinations.generated.deleteaccount.destinations.DeleteAccountDestination
import com.ramcosta.composedestinations.generated.destinations.NoDaemonDestination
import com.ramcosta.composedestinations.generated.filter.destinations.FilterDestination
import com.ramcosta.composedestinations.generated.home.destinations.Android16UpgradeWarningInfoDestination
import com.ramcosta.composedestinations.generated.home.destinations.ConnectDestination
import com.ramcosta.composedestinations.generated.home.destinations.DeviceNameInfoDestination
import com.ramcosta.composedestinations.generated.home.destinations.DeviceRevokedDestination
import com.ramcosta.composedestinations.generated.home.destinations.OutOfTimeDestination
import com.ramcosta.composedestinations.generated.home.destinations.WelcomeDestination
import com.ramcosta.composedestinations.generated.location.destinations.SearchLocationDestination
import com.ramcosta.composedestinations.generated.location.destinations.SelectLocationDestination
import com.ramcosta.composedestinations.generated.managedevices.destinations.ManageDevicesDestination
import com.ramcosta.composedestinations.generated.managedevices.destinations.ManageDevicesRemoveConfirmationDestination
import com.ramcosta.composedestinations.generated.multihop.destinations.MultihopDestination
import com.ramcosta.composedestinations.generated.notification.destinations.NotificationSettingsDestination
import com.ramcosta.composedestinations.generated.problemreport.destinations.ReportProblemDestination
import com.ramcosta.composedestinations.generated.problemreport.destinations.ReportProblemNoEmailDestination
import com.ramcosta.composedestinations.generated.problemreport.destinations.ViewLogsDestination
import com.ramcosta.composedestinations.generated.redeemvoucher.destinations.RedeemVoucherDestination
import com.ramcosta.composedestinations.generated.serveripoverride.destinations.ImportOverridesByTextDestination
import com.ramcosta.composedestinations.generated.serveripoverride.destinations.ResetServerIpOverridesConfirmationDestination
import com.ramcosta.composedestinations.generated.serveripoverride.destinations.ServerIpOverridesDestination
import com.ramcosta.composedestinations.generated.serveripoverride.destinations.ServerIpOverridesInfoDestination
import com.ramcosta.composedestinations.generated.settings.destinations.SettingsDestination
import com.ramcosta.composedestinations.generated.splittunneling.destinations.SplitTunnelingDestination
import com.ramcosta.composedestinations.generated.vpnsettings.destinations.ConnectOnStartupInfoDestination
import com.ramcosta.composedestinations.generated.vpnsettings.destinations.ContentBlockersInfoDestination
import com.ramcosta.composedestinations.generated.vpnsettings.destinations.CustomDnsInfoDestination
import com.ramcosta.composedestinations.generated.vpnsettings.destinations.DeviceIpInfoDestination
import com.ramcosta.composedestinations.generated.vpnsettings.destinations.DnsDestination
import com.ramcosta.composedestinations.generated.vpnsettings.destinations.Ipv6InfoDestination
import com.ramcosta.composedestinations.generated.vpnsettings.destinations.LocalNetworkSharingInfoDestination
import com.ramcosta.composedestinations.generated.vpnsettings.destinations.MalwareInfoDestination
import com.ramcosta.composedestinations.generated.vpnsettings.destinations.MtuDestination
import com.ramcosta.composedestinations.generated.vpnsettings.destinations.QuantumResistanceInfoDestination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import com.ramcosta.composedestinations.rememberNavHostEngine
import com.ramcosta.composedestinations.utils.rememberDestinationsNavigator
import kotlinx.coroutines.cancel
import net.mullvad.mullvadvpn.common.compose.LocalSharedTransitionScope
import net.mullvad.mullvadvpn.common.compose.accessibilityDataSensitive
import net.mullvad.mullvadvpn.core.nav3.LocalResultStore
import net.mullvad.mullvadvpn.core.nav3.Navigator
import net.mullvad.mullvadvpn.core.nav3.SplashNavKey
import net.mullvad.mullvadvpn.core.nav3.rememberNavigationState
import net.mullvad.mullvadvpn.core.nav3.rememberResultStore
import net.mullvad.mullvadvpn.core.nav3.toEntries
import net.mullvad.mullvadvpn.feature.account.impl.navigation.accountEntry
import net.mullvad.mullvadvpn.feature.addtime.impl.navigation.addTimeEntry
import net.mullvad.mullvadvpn.feature.anticensorship.impl.navigation.anticensorshipEntry
import net.mullvad.mullvadvpn.feature.apiaccess.impl.navigation.apiAccessEntry
import net.mullvad.mullvadvpn.feature.appearance.impl.navigation.appearanceEntry
import net.mullvad.mullvadvpn.feature.appinfo.impl.navigation.appInfoEntry
import net.mullvad.mullvadvpn.feature.autoconnect.impl.navigation.autoConnectEntry
import net.mullvad.mullvadvpn.feature.customlist.impl.navigation.createCustomListEntry
import net.mullvad.mullvadvpn.feature.customlist.impl.navigation.customListsEntry
import net.mullvad.mullvadvpn.feature.customlist.impl.navigation.deleteCustomListEntry
import net.mullvad.mullvadvpn.feature.customlist.impl.navigation.deleteCustomListNameEntry
import net.mullvad.mullvadvpn.feature.customlist.impl.navigation.discardCustomListEntry
import net.mullvad.mullvadvpn.feature.customlist.impl.navigation.editCustomListEntry
import net.mullvad.mullvadvpn.feature.customlist.impl.navigation.editCustomListNameEntry
import net.mullvad.mullvadvpn.feature.daita.impl.navigation.daitaEntry
import net.mullvad.mullvadvpn.feature.filter.impl.navigation.filterEntry
import net.mullvad.mullvadvpn.feature.home.impl.navigation.connectEntry
import net.mullvad.mullvadvpn.feature.home.impl.navigation.deviceRevokedEntry
import net.mullvad.mullvadvpn.feature.home.impl.navigation.outOfTimeEntry
import net.mullvad.mullvadvpn.feature.home.impl.navigation.welcomeEntry
import net.mullvad.mullvadvpn.feature.location.impl.navigation.selectLocationEntry
import net.mullvad.mullvadvpn.feature.login.impl.devicelist.navigation.deviceListEntry
import net.mullvad.mullvadvpn.feature.login.impl.devicelist.navigation.removeDeviceConfirmationDialogEntry
import net.mullvad.mullvadvpn.feature.login.impl.navigation.apiUnreachableEntry
import net.mullvad.mullvadvpn.feature.login.impl.navigation.createAccountConfirmationEntry
import net.mullvad.mullvadvpn.feature.login.impl.navigation.loginEntry
import net.mullvad.mullvadvpn.feature.managedevices.impl.navigation.manageDevicesEntry
import net.mullvad.mullvadvpn.feature.multihop.impl.navigation.multihopEntry
import net.mullvad.mullvadvpn.feature.notification.impl.navigation.notificationEntry
import net.mullvad.mullvadvpn.feature.problemreport.impl.navigation.problemReportEntry
import net.mullvad.mullvadvpn.feature.redeemvoucher.impl.navigation.redeemVoucherEntry
import net.mullvad.mullvadvpn.feature.serveripoverride.impl.navigation.serverIpOverrideEntry
import net.mullvad.mullvadvpn.feature.settings.impl.navigation.settingsEntry
import net.mullvad.mullvadvpn.feature.splittunneling.impl.navigation.splitTunnelingEntry
import net.mullvad.mullvadvpn.feature.vpnsettings.impl.navigation.vpnSettingsEntry
import net.mullvad.mullvadvpn.screen.privacy.navigation.privacyDisclaimerEntry
import net.mullvad.mullvadvpn.screen.splash.navigation.splashEntry
import net.mullvad.mullvadvpn.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.serviceconnection.ServiceConnectionState
import net.mullvad.mullvadvpn.util.BackstackObserver
import org.koin.androidx.compose.koinViewModel

@NavHostGraph
annotation class MainGraph {
    @ExternalDestination<AccountDestination>
    @ExternalDestination<Android16UpgradeWarningInfoDestination>
    @ExternalDestination<AntiCensorshipSettingsDestination>
    @ExternalDestination<ApiAccessListDestination>
    @ExternalDestination<ApiAccessMethodDetailsDestination>
    @ExternalDestination<ApiAccessMethodInfoDestination>
    @ExternalDestination<AppearanceDestination>
    @ExternalDestination<AppInfoDestination>
    @ExternalDestination<AutoConnectAndLockdownModeDestination>
    @ExternalDestination<ChangelogDestination>
    @ExternalDestination<ConnectDestination>
    @ExternalDestination<ConnectOnStartupInfoDestination>
    @ExternalDestination<ContentBlockersInfoDestination>
    @ExternalDestination<CreateCustomListDestination>
    @ExternalDestination<CustomDnsInfoDestination>
    @ExternalDestination<CustomListLocationsDestination>
    @ExternalDestination<CustomListsDestination>
    @ExternalDestination<CustomPortDestination>
    @ExternalDestination<DaitaDestination>
    @ExternalDestination<DaitaDirectOnlyConfirmationDestination>
    @ExternalDestination<DaitaDirectOnlyInfoDestination>
    @ExternalDestination<DeleteAccountDestination>
    @ExternalDestination<DeleteAccountConfirmationDestination>
    @ExternalDestination<DeleteAccountCompleteDestination>
    @ExternalDestination<DeleteApiAccessMethodConfirmationDestination>
    @ExternalDestination<DeleteCustomListDestination>
    @ExternalDestination<DeviceIpInfoDestination>
    @ExternalDestination<DeviceNameInfoDestination>
    @ExternalDestination<DeviceRevokedDestination>
    @ExternalDestination<DiscardApiAccessChangesDestination>
    @ExternalDestination<DiscardChangesDestination>
    @ExternalDestination<DnsDestination>
    @ExternalDestination<EditApiAccessMethodDestination>
    @ExternalDestination<EditCustomListDestination>
    @ExternalDestination<EditCustomListNameDestination>
    @ExternalDestination<EncryptedDnsProxyInfoDestination>
    @ExternalDestination<FilterDestination>
    @ExternalDestination<ImportOverridesByTextDestination>
    @ExternalDestination<Ipv6InfoDestination>
    @ExternalDestination<LocalNetworkSharingInfoDestination>
    @ExternalDestination<MalwareInfoDestination>
    @ExternalDestination<ManageDevicesDestination>
    @ExternalDestination<ManageDevicesRemoveConfirmationDestination>
    @ExternalDestination<MtuDestination>
    @ExternalDestination<MultihopDestination>
    @ExternalDestination<NotificationSettingsDestination>
    @ExternalDestination<OutOfTimeDestination>
    @ExternalDestination<QuantumResistanceInfoDestination>
    @ExternalDestination<RedeemVoucherDestination>
    @ExternalDestination<ReportProblemDestination>
    @ExternalDestination<ReportProblemNoEmailDestination>
    @ExternalDestination<ResetServerIpOverridesConfirmationDestination>
    @ExternalDestination<SaveApiAccessMethodDestination>
    @ExternalDestination<SearchLocationDestination>
    @ExternalDestination<SelectLocationDestination>
    @ExternalDestination<SelectPortDestination>
    @ExternalDestination<ServerIpOverridesDestination>
    @ExternalDestination<ServerIpOverridesInfoDestination>
    @ExternalDestination<SettingsDestination>
    @ExternalDestination<SplitTunnelingDestination>
    @ExternalDestination<VerificationPendingDestination>
    @ExternalDestination<ViewLogsDestination>
    @ExternalDestination<WelcomeDestination>
    companion object Includes
}

@OptIn(
    ExperimentalComposeUiApi::class,
    ExperimentalSharedTransitionApi::class,
    ExperimentalPermissionsApi::class,
)
@Composable
fun MullvadApp(
    backstackObserver: BackstackObserver,
    serviceConnectionManager: ServiceConnectionManager,
) {
    val engine = rememberNavHostEngine()
    val navHostController: NavHostController = engine.rememberNavController()
    val navigator: DestinationsNavigator = navHostController.rememberDestinationsNavigator()

    val navigationState = rememberNavigationState(SplashNavKey)
    val nav3 = remember { Navigator(navigationState) }
    val resultStore = rememberResultStore()

    val mullvadAppViewModel = koinViewModel<MullvadAppViewModel>()

    DisposableEffect(Unit) {
        backstackObserver.addOnDestinationChangedListener(navHostController)
        onDispose { backstackObserver.removeOnDestinationChangedListener(navHostController) }
    }

    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
        CheckNotificationPermission(serviceConnectionManager)
    }

    val entryProvider = entryProvider {
        accountEntry(nav3)
        addTimeEntry(nav3)
        anticensorshipEntry(nav3)
        apiAccessEntry(nav3)
        apiUnreachableEntry(nav3)
        appearanceEntry(nav3)
        appInfoEntry(nav3)
        autoConnectEntry(nav3)
        deviceListEntry(nav3)
        connectEntry(nav3)
        createAccountConfirmationEntry(nav3)
        daitaEntry(nav3)
        deviceRevokedEntry(nav3)
        filterEntry(nav3)
        loginEntry(nav3)
        outOfTimeEntry(nav3)
        manageDevicesEntry(nav3)
        multihopEntry(nav3)
        notificationEntry(nav3)
        privacyDisclaimerEntry(nav3)
        problemReportEntry(nav3)
        redeemVoucherEntry(nav3)
        removeDeviceConfirmationDialogEntry(nav3)
        selectLocationEntry(nav3)
        serverIpOverrideEntry(nav3)
        settingsEntry(nav3)
        splitTunnelingEntry(nav3)
        vpnSettingsEntry(nav3)
        welcomeEntry(nav3)

        customListsEntry(nav3)
        editCustomListEntry(nav3)
        deleteCustomListEntry(nav3)
        discardCustomListEntry(nav3)
        createCustomListEntry(nav3)
        deleteCustomListNameEntry(nav3)
        editCustomListNameEntry(nav3)

        splashEntry(nav3)
    }

    SharedTransitionLayout {
        CompositionLocalProvider(LocalSharedTransitionScope provides this@SharedTransitionLayout) {
            CompositionLocalProvider(LocalResultStore provides resultStore) {
                NavDisplay(
                    modifier =
                        Modifier.semantics { testTagsAsResourceId = true }
                            .fillMaxSize()
                            .accessibilityDataSensitive(),
                    sceneStrategies = listOf(DialogSceneStrategy(), SinglePaneSceneStrategy()),
                    entries = navigationState.toEntries(entryProvider),
                    onBack = { nav3.goBack() },
                    sharedTransitionScope = this@SharedTransitionLayout,
                )
            }
        }
    }

    // For the following LaunchedEffect we do not use CollectSideEffectWithLifecycle since we
    // collect from StateFlow/SharedFlow with replay and don't want to trigger a navigation again.

    // Globally handle daemon dropped connection with NoDaemonScreen
    LaunchedEffect(Unit) {
        mullvadAppViewModel.uiSideEffect.collect {
            Logger.i { "DaemonScreenEvent: $it" }
            when (it) {
                DaemonScreenEvent.Show ->
                    navigator.navigate(NoDaemonDestination) { launchSingleTop = true }

                DaemonScreenEvent.Remove -> navigator.popBackStack(NoDaemonDestination, true)
            }
        }
    }
}

@OptIn(ExperimentalPermissionsApi::class)
@Composable
@RequiresApi(Build.VERSION_CODES.TIRAMISU)
private fun CheckNotificationPermission(serviceConnectionManager: ServiceConnectionManager) {
    val notificationPermission =
        rememberPermissionState(permission = Manifest.permission.POST_NOTIFICATIONS)
    LaunchedEffect(Unit) {
        serviceConnectionManager.connectionState.collect {
            if (it is ServiceConnectionState.Bound) {
                if (!notificationPermission.status.isGranted) {
                    notificationPermission.launchPermissionRequest()
                    cancel(
                        message =
                            "We should only show one notification permission dialog per app start"
                    )
                }
            }
        }
    }
}
