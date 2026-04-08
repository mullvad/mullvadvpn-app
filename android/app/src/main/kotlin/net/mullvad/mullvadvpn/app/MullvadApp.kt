@file:Suppress("MatchingDeclarationName")

package net.mullvad.mullvadvpn.app

import android.Manifest
import android.os.Build
import androidx.annotation.RequiresApi
import androidx.compose.animation.ContentTransform
import androidx.compose.animation.ExperimentalSharedTransitionApi
import androidx.compose.animation.SharedTransitionLayout
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.semantics.testTagsAsResourceId
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.compose.LocalLifecycleOwner
import androidx.lifecycle.repeatOnLifecycle
import androidx.navigation3.runtime.entryProvider
import androidx.navigation3.scene.DialogSceneStrategy
import androidx.navigation3.scene.SinglePaneSceneStrategy
import androidx.navigation3.ui.NavDisplay
import co.touchlab.kermit.Logger
import com.google.accompanist.permissions.ExperimentalPermissionsApi
import com.google.accompanist.permissions.isGranted
import com.google.accompanist.permissions.rememberPermissionState
import kotlinx.coroutines.cancel
import net.mullvad.mullvadvpn.common.compose.LocalSharedTransitionScope
import net.mullvad.mullvadvpn.common.compose.accessibilityDataSensitive
import net.mullvad.mullvadvpn.core.LocalResultStore
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.core.animation.TRANSITION_DEFAULT_DURATION_MS
import net.mullvad.mullvadvpn.core.rememberNavigationState
import net.mullvad.mullvadvpn.core.rememberResultStore
import net.mullvad.mullvadvpn.core.scene.SingleOverlaySceneStrategy
import net.mullvad.mullvadvpn.core.scene.rememberListDetailSceneStrategy
import net.mullvad.mullvadvpn.core.toEntries
import net.mullvad.mullvadvpn.feature.account.impl.navigation.accountEntry
import net.mullvad.mullvadvpn.feature.addtime.impl.navigation.addTimeVerificationPendingEntry
import net.mullvad.mullvadvpn.feature.anticensorship.impl.navigation.anticensorshipEntry
import net.mullvad.mullvadvpn.feature.apiaccess.impl.navigation.apiAccessEntry
import net.mullvad.mullvadvpn.feature.appearance.impl.navigation.appearanceEntry
import net.mullvad.mullvadvpn.feature.appinfo.impl.navigation.changelogEntry
import net.mullvad.mullvadvpn.feature.autoconnect.impl.navigation.autoConnectEntry
import net.mullvad.mullvadvpn.feature.customlist.impl.navigation.customListEntry
import net.mullvad.mullvadvpn.feature.daita.impl.navigation.daitaEntry
import net.mullvad.mullvadvpn.feature.deleteaccount.impl.navigation.deleteAccountEntry
import net.mullvad.mullvadvpn.feature.filter.impl.navigation.filterEntry
import net.mullvad.mullvadvpn.feature.home.impl.navigation.homeEntry
import net.mullvad.mullvadvpn.feature.location.impl.navigation.selectLocationEntry
import net.mullvad.mullvadvpn.feature.login.impl.devicelist.navigation.deviceListEntry
import net.mullvad.mullvadvpn.feature.login.impl.devicelist.navigation.removeDeviceConfirmationDialogEntry
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
import net.mullvad.mullvadvpn.screen.navigation.NoDaemonNavKey
import net.mullvad.mullvadvpn.screen.navigation.SplashNavKey
import net.mullvad.mullvadvpn.screen.navigation.noDaemonEntry
import net.mullvad.mullvadvpn.screen.navigation.privacyDisclaimerEntry
import net.mullvad.mullvadvpn.screen.navigation.splashEntry
import net.mullvad.mullvadvpn.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.serviceconnection.ServiceConnectionState
import org.koin.androidx.compose.koinViewModel

@OptIn(
    ExperimentalComposeUiApi::class,
    ExperimentalSharedTransitionApi::class,
    ExperimentalPermissionsApi::class,
)
@Composable
@Suppress("LongMethod")
fun MullvadApp(serviceConnectionManager: ServiceConnectionManager) {
    val resultStore = rememberResultStore()
    val navigationState = rememberNavigationState(SplashNavKey)

    val listDetailStrategy = rememberListDetailSceneStrategy<NavKey2>()
    val dialogStrategy = remember { DialogSceneStrategy<NavKey2>() }
    val bottomSheetStrategy = remember { SingleOverlaySceneStrategy<NavKey2>() }
    val singlePaneStrategy = remember { SinglePaneSceneStrategy<NavKey2>() }

    val nav3 = remember {
        Navigator(
            state = navigationState,
            resultStore = resultStore,
            screenIsListDetailTargetWidth = listDetailStrategy.isListDetailTargetWidth(),
        )
    }

    val mullvadAppViewModel = koinViewModel<MullvadAppViewModel>()

    val lifecycleOwner = LocalLifecycleOwner.current
    LaunchedEffect(lifecycleOwner) {
        lifecycleOwner.lifecycle.repeatOnLifecycle(Lifecycle.State.STARTED) {
            navigationState.backStackFlow.collect { backstack ->
                mullvadAppViewModel.setCurrentBackStack(backstack)
            }
        }
    }

    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
        CheckNotificationPermission(serviceConnectionManager)
    }

    val entryProvider = entryProvider {
        accountEntry(nav3)
        addTimeVerificationPendingEntry(nav3)
        anticensorshipEntry(nav3)
        apiAccessEntry(nav3)
        appearanceEntry(nav3)
        autoConnectEntry(nav3)
        changelogEntry(nav3)
        customListEntry(nav3)
        daitaEntry(nav3)
        deleteAccountEntry(nav3)
        deviceListEntry(nav3)
        filterEntry(nav3)
        homeEntry(nav3)
        loginEntry(nav3)
        manageDevicesEntry(nav3)
        multihopEntry(nav3)
        noDaemonEntry(nav3)
        notificationEntry(nav3)
        privacyDisclaimerEntry(nav3)
        problemReportEntry(nav3)
        redeemVoucherEntry(nav3)
        removeDeviceConfirmationDialogEntry(nav3)
        selectLocationEntry(nav3)
        serverIpOverrideEntry(nav3)
        settingsEntry(nav3)
        splashEntry(nav3)
        splitTunnelingEntry(nav3)
        vpnSettingsEntry(nav3)
    }

    SharedTransitionLayout {
        CompositionLocalProvider(LocalSharedTransitionScope provides this@SharedTransitionLayout) {
            CompositionLocalProvider(LocalResultStore provides resultStore) {
                NavDisplay(
                    modifier =
                        Modifier.semantics { testTagsAsResourceId = true }
                            .fillMaxSize()
                            .accessibilityDataSensitive(),
                    sceneStrategies =
                        listOf(
                            listDetailStrategy,
                            dialogStrategy,
                            bottomSheetStrategy,
                            singlePaneStrategy,
                        ),
                    entries = navigationState.toEntries(entryProvider),
                    onBack = { nav3.goBack() },
                    sharedTransitionScope = this@SharedTransitionLayout,
                    transitionSpec = { defaultNavDisplayTransitionSpec() },
                    popTransitionSpec = { defaultNavDisplayTransitionSpec() },
                    predictivePopTransitionSpec = { defaultNavDisplayTransitionSpec() },
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
                DaemonScreenEvent.Show -> nav3.navigate(NoDaemonNavKey)

                DaemonScreenEvent.Remove -> nav3.goBackUntil(NoDaemonNavKey, inclusive = true)
            }
        }
    }
}

private fun defaultNavDisplayTransitionSpec(): ContentTransform =
    fadeIn(tween(TRANSITION_DEFAULT_DURATION_MS)) togetherWith
        fadeOut(tween(TRANSITION_DEFAULT_DURATION_MS))

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
