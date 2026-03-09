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
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.semantics.testTagsAsResourceId
import androidx.navigation.NavHostController
import androidx.navigation3.runtime.entryProvider
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
import com.ramcosta.composedestinations.generated.login.destinations.CreateAccountConfirmationDestination
import com.ramcosta.composedestinations.generated.login.destinations.DeviceListDestination
import com.ramcosta.composedestinations.generated.login.destinations.LoginDestination
import com.ramcosta.composedestinations.generated.login.destinations.RemoveDeviceConfirmationDestination
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
import net.mullvad.mullvadvpn.core.nav3.ResultStore
import net.mullvad.mullvadvpn.core.nav3.SplashNavKey
import net.mullvad.mullvadvpn.core.nav3.rememberNavigationState
import net.mullvad.mullvadvpn.core.nav3.rememberResultStore
import net.mullvad.mullvadvpn.core.nav3.toEntries
import net.mullvad.mullvadvpn.feature.home.impl.navigation.connectEntry
import net.mullvad.mullvadvpn.feature.login.impl.navigation.apiUnreachableEntry
import net.mullvad.mullvadvpn.feature.login.impl.navigation.createAccountConfirmationEntry
import net.mullvad.mullvadvpn.feature.login.impl.navigation.loginEntry
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
    @ExternalDestination<CreateAccountConfirmationDestination>
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
    @ExternalDestination<DeviceListDestination>
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
    @ExternalDestination<LoginDestination>
    @ExternalDestination<MalwareInfoDestination>
    @ExternalDestination<ManageDevicesDestination>
    @ExternalDestination<ManageDevicesRemoveConfirmationDestination>
    @ExternalDestination<MtuDestination>
    @ExternalDestination<MultihopDestination>
    @ExternalDestination<NotificationSettingsDestination>
    @ExternalDestination<OutOfTimeDestination>
    @ExternalDestination<QuantumResistanceInfoDestination>
    @ExternalDestination<RedeemVoucherDestination>
    @ExternalDestination<RemoveDeviceConfirmationDestination>
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
    val navigator3 = remember { Navigator(navigationState) }
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
        splashEntry(navigator3)
        loginEntry(navigator3)
        connectEntry(navigator3)
        createAccountConfirmationEntry(navigator3)
        apiUnreachableEntry(navigator3)
        vpnSettingsEntry(navigator3)
        privacyDisclaimerEntry(navigator3)
    }

    SharedTransitionLayout {
        CompositionLocalProvider(LocalSharedTransitionScope provides this@SharedTransitionLayout) {
            CompositionLocalProvider(LocalResultStore provides resultStore) {
                NavDisplay(
                    modifier =
                        Modifier.semantics { testTagsAsResourceId = true }
                            .fillMaxSize()
                            .accessibilityDataSensitive(),
                    entries = navigationState.toEntries(entryProvider),
                    onBack = { navigator3.goBack() },
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
