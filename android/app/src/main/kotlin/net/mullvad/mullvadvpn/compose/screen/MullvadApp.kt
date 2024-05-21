package net.mullvad.mullvadvpn.compose.screen

import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.semantics.testTagsAsResourceId
import androidx.navigation.NavHostController
import com.ramcosta.composedestinations.DestinationsNavHost
import com.ramcosta.composedestinations.navigation.navigate
import com.ramcosta.composedestinations.navigation.popBackStack
import com.ramcosta.composedestinations.rememberNavHostEngine
import com.ramcosta.composedestinations.utils.destination
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.NavGraphs
import net.mullvad.mullvadvpn.compose.destinations.ChangelogDestination
import net.mullvad.mullvadvpn.compose.destinations.ConnectDestination
import net.mullvad.mullvadvpn.compose.destinations.NoDaemonScreenDestination
import net.mullvad.mullvadvpn.compose.destinations.OutOfTimeDestination
import net.mullvad.mullvadvpn.compose.util.LaunchedEffectCollect
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
    val navController: NavHostController = engine.rememberNavController()

    val serviceVm = koinViewModel<NoDaemonViewModel>()
    val permissionVm = koinViewModel<VpnPermissionViewModel>()

    DisposableEffect(Unit) {
        navController.addOnDestinationChangedListener(serviceVm)
        onDispose { navController.removeOnDestinationChangedListener(serviceVm) }
    }

    DestinationsNavHost(
        modifier = Modifier.semantics { testTagsAsResourceId = true }.fillMaxSize(),
        engine = engine,
        navController = navController,
        navGraph = NavGraphs.root,
    )

    // Globally handle daemon dropped connection with NoDaemonScreen
    LaunchedEffectCollect(serviceVm.uiSideEffect) {
        when (it) {
            DaemonScreenEvent.Show ->
                navController.navigate(NoDaemonScreenDestination) { launchSingleTop = true }
            DaemonScreenEvent.Remove -> navController.popBackStack(NoDaemonScreenDestination, true)
        }
    }

    // Globally show the changelog
    val changeLogsViewModel = koinViewModel<ChangelogViewModel>()
    LaunchedEffectCollect(changeLogsViewModel.uiSideEffect) {

        // Wait until we are in an acceptable destination
        navController.currentBackStackEntryFlow
            .map { it.destination() }
            .first { it in changeLogDestinations }

        navController.navigate(ChangelogDestination(it).route)
    }

    // Ask for VPN Permission
    val launchVpnPermission =
        rememberLauncherForActivityResult(RequestVpnPermission()) { result ->
            if (result) {
                permissionVm.connect()
            }
        }
    LaunchedEffectCollect(permissionVm.uiSideEffect) {
        if (it is VpnPermissionSideEffect.ShowDialog) {
            launchVpnPermission.launch(Unit)
        }
    }
}
