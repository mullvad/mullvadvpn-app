@file:Suppress("MatchingDeclarationName")

package net.mullvad.mullvadvpn.app

import androidx.compose.animation.ExperimentalSharedTransitionApi
import androidx.compose.animation.SharedTransitionLayout
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.semantics.testTagsAsResourceId
import androidx.navigation.NavHostController
import co.touchlab.kermit.Logger
import com.ramcosta.composedestinations.DestinationsNavHost
import com.ramcosta.composedestinations.annotation.ExternalDestination
import com.ramcosta.composedestinations.annotation.NavHostGraph
import com.ramcosta.composedestinations.generated.NavGraphs
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
import com.ramcosta.composedestinations.generated.login.destinations.ApiUnreachableInfoDestination
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
import com.ramcosta.composedestinations.generated.vpnsettings.destinations.VpnSettingsDestination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import com.ramcosta.composedestinations.navigation.dependency
import com.ramcosta.composedestinations.rememberNavHostEngine
import com.ramcosta.composedestinations.utils.rememberDestinationsNavigator
import net.mullvad.mullvadvpn.common.compose.LocalSharedTransitionScope
import net.mullvad.mullvadvpn.common.compose.accessibilityDataSensitive
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
    @ExternalDestination<ApiUnreachableInfoDestination>
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
    @ExternalDestination<VpnSettingsDestination>
    @ExternalDestination<WelcomeDestination>
    companion object Includes
}

@OptIn(ExperimentalComposeUiApi::class, ExperimentalSharedTransitionApi::class)
@Composable
fun MullvadApp(backstackObserver: BackstackObserver) {
    val engine = rememberNavHostEngine()
    val navHostController: NavHostController = engine.rememberNavController()
    val navigator: DestinationsNavigator = navHostController.rememberDestinationsNavigator()

    val mullvadAppViewModel = koinViewModel<MullvadAppViewModel>()

    DisposableEffect(Unit) {
        backstackObserver.addOnDestinationChangedListener(navHostController)
        onDispose { backstackObserver.removeOnDestinationChangedListener(navHostController) }
    }

    SharedTransitionLayout {
        CompositionLocalProvider(LocalSharedTransitionScope provides this@SharedTransitionLayout) {
            DestinationsNavHost(
                modifier =
                    Modifier.semantics { testTagsAsResourceId = true }
                        .fillMaxSize()
                        .accessibilityDataSensitive(),
                engine = engine,
                navController = navHostController,
                navGraph = NavGraphs.main,
                dependenciesContainerBuilder = { dependency(this@SharedTransitionLayout) },
            )
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
