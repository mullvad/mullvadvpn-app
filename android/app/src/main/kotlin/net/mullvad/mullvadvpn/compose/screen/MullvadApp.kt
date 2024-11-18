package net.mullvad.mullvadvpn.compose.screen

import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.semantics.testTagsAsResourceId
import androidx.navigation.NavHostController
import arrow.core.merge
import co.touchlab.kermit.Logger
import com.ramcosta.composedestinations.DestinationsNavHost
import com.ramcosta.composedestinations.generated.NavGraphs
import com.ramcosta.composedestinations.generated.destinations.NoDaemonDestination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import com.ramcosta.composedestinations.rememberNavHostEngine
import com.ramcosta.composedestinations.utils.rememberDestinationsNavigator
import net.mullvad.mullvadvpn.compose.util.CreateVpnProfile
import net.mullvad.mullvadvpn.lib.common.util.prepareVpnSafe
import net.mullvad.mullvadvpn.lib.model.PrepareError
import net.mullvad.mullvadvpn.lib.model.Prepared
import net.mullvad.mullvadvpn.viewmodel.DaemonScreenEvent
import net.mullvad.mullvadvpn.viewmodel.NoDaemonViewModel
import net.mullvad.mullvadvpn.viewmodel.VpnProfileSideEffect
import net.mullvad.mullvadvpn.viewmodel.VpnProfileViewModel
import org.koin.androidx.compose.koinViewModel

@OptIn(ExperimentalComposeUiApi::class)
@Composable
fun MullvadApp() {
    val engine = rememberNavHostEngine()
    val navHostController: NavHostController = engine.rememberNavController()
    val navigator: DestinationsNavigator = navHostController.rememberDestinationsNavigator()

    val serviceVm = koinViewModel<NoDaemonViewModel>()
    val permissionVm = koinViewModel<VpnProfileViewModel>()

    DisposableEffect(Unit) {
        navHostController.addOnDestinationChangedListener(serviceVm)
        onDispose { navHostController.removeOnDestinationChangedListener(serviceVm) }
    }

    DestinationsNavHost(
        modifier = Modifier.semantics { testTagsAsResourceId = true }.fillMaxSize(),
        engine = engine,
        navController = navHostController,
        navGraph = NavGraphs.root,
    )

    // For the following LaunchedEffect we do not use CollectSideEffectWithLifecycle since we
    // collect from StateFlow/SharedFlow with replay and don't want to trigger a navigation again.

    // Globally handle daemon dropped connection with NoDaemonScreen
    LaunchedEffect(Unit) {
        serviceVm.uiSideEffect.collect {
            Logger.i { "DaemonScreenEvent: $it" }
            when (it) {
                DaemonScreenEvent.Show ->
                    navigator.navigate(NoDaemonDestination) { launchSingleTop = true }

                DaemonScreenEvent.Remove -> navigator.popBackStack(NoDaemonDestination, true)
            }
        }
    }

    // Ask for VPN Permission
    val launchVpnPermission =
        rememberLauncherForActivityResult(CreateVpnProfile()) { _ -> permissionVm.connect() }
    val context = LocalContext.current
    LaunchedEffect(Unit) {
        permissionVm.uiSideEffect.collect {
            if (it is VpnProfileSideEffect.RequestVpnProfile) {
                val prepareResult = context.prepareVpnSafe().merge()
                when (prepareResult) {
                    is PrepareError.NotPrepared ->
                        launchVpnPermission.launch(prepareResult.prepareIntent)
                    // If legacy or other always on connect at let daemon generate a error state
                    is PrepareError.OtherLegacyAlwaysOnVpn,
                    is PrepareError.OtherAlwaysOnApp,
                    Prepared -> permissionVm.connect()
                }
            }
        }
    }
}
