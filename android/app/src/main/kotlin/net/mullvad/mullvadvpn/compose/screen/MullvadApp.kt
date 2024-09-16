package net.mullvad.mullvadvpn.compose.screen

import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.semantics.testTagsAsResourceId
import androidx.navigation.NavHostController
import co.touchlab.kermit.Logger
import com.ramcosta.composedestinations.DestinationsNavHost
import com.ramcosta.composedestinations.generated.NavGraphs
import com.ramcosta.composedestinations.generated.destinations.ChangelogDestination
import com.ramcosta.composedestinations.generated.destinations.ConnectDestination
import com.ramcosta.composedestinations.generated.destinations.NoDaemonDestination
import com.ramcosta.composedestinations.generated.destinations.OutOfTimeDestination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import com.ramcosta.composedestinations.rememberNavHostEngine
import com.ramcosta.composedestinations.utils.destination
import com.ramcosta.composedestinations.utils.rememberDestinationsNavigator
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.map
import net.mullvad.mullvadvpn.compose.util.RequestVpnPermission
import net.mullvad.mullvadvpn.viewmodel.ChangelogViewModel
import net.mullvad.mullvadvpn.viewmodel.DaemonScreenEvent
import net.mullvad.mullvadvpn.viewmodel.NoDaemonViewModel
import net.mullvad.mullvadvpn.viewmodel.VpnPermissionSideEffect
import net.mullvad.mullvadvpn.viewmodel.VpnPermissionViewModel
import org.koin.androidx.compose.koinViewModel

private val changeLogDestinations = listOf(ConnectDestination, OutOfTimeDestination)

@OptIn(ExperimentalComposeUiApi::class)
@Composable
fun MullvadApp() {
    val engine = rememberNavHostEngine()
    val navHostController: NavHostController = engine.rememberNavController()
    val navigator: DestinationsNavigator = navHostController.rememberDestinationsNavigator()

    val serviceVm = koinViewModel<NoDaemonViewModel>()
    val permissionVm = koinViewModel<VpnPermissionViewModel>()

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

    // Globally show the changelog
    val changeLogsViewModel = koinViewModel<ChangelogViewModel>()
    LaunchedEffect(Unit) {
        changeLogsViewModel.uiSideEffect.collect {
            // Wait until we are in an acceptable destination
            navHostController.currentBackStackEntryFlow
                .map { it.destination() }
                .first { it in changeLogDestinations }

            navigator.navigate(ChangelogDestination(it))
        }
    }

    // Ask for VPN Permission
    val launchVpnPermission =
        rememberLauncherForActivityResult(RequestVpnPermission()) { _ -> permissionVm.connect() }
    LaunchedEffect(Unit) {
        permissionVm.uiSideEffect.collect {
            if (it is VpnPermissionSideEffect.ShowDialog) {
                launchVpnPermission.launch(Unit)
            }
        }
    }
}
