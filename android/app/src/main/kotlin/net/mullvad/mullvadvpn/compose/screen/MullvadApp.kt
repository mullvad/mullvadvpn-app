package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.semantics.testTagsAsResourceId
import androidx.navigation.NavController
import androidx.navigation.NavHostController
import com.ramcosta.composedestinations.DestinationsNavHost
import com.ramcosta.composedestinations.navigation.navigate
import com.ramcosta.composedestinations.navigation.popBackStack
import com.ramcosta.composedestinations.rememberNavHostEngine
import com.ramcosta.composedestinations.utils.destination
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.dropWhile
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.map
import net.mullvad.mullvadvpn.compose.NavGraphs
import net.mullvad.mullvadvpn.compose.appCurrentDestinationFlow
import net.mullvad.mullvadvpn.compose.destinations.ChangelogDestination
import net.mullvad.mullvadvpn.compose.destinations.ConnectDestination
import net.mullvad.mullvadvpn.compose.destinations.Destination
import net.mullvad.mullvadvpn.compose.destinations.NoDaemonScreenDestination
import net.mullvad.mullvadvpn.compose.destinations.OutOfTimeDestination
import net.mullvad.mullvadvpn.compose.destinations.PrivacyDisclaimerDestination
import net.mullvad.mullvadvpn.compose.destinations.SplashDestination
import net.mullvad.mullvadvpn.viewmodel.ChangelogViewModel
import net.mullvad.mullvadvpn.viewmodel.ServiceConnectionViewModel
import net.mullvad.mullvadvpn.viewmodel.ServiceState
import org.koin.androidx.compose.koinViewModel

private val changeLogDestinations = listOf(ConnectDestination, OutOfTimeDestination)
private val noServiceDestinations = listOf(SplashDestination, PrivacyDisclaimerDestination)

@OptIn(ExperimentalComposeUiApi::class)
@Composable
fun MullvadApp() {
    val engine = rememberNavHostEngine()
    val navController: NavHostController = engine.rememberNavController()

    val serviceVm = koinViewModel<ServiceConnectionViewModel>()

    DestinationsNavHost(
        modifier = Modifier.semantics { testTagsAsResourceId = true }.fillMaxSize(),
        engine = engine,
        navController = navController,
        navGraph = NavGraphs.root
    )

    // Globally handle daemon dropped connection with NoDaemonScreen
    LaunchedEffect(Unit) {
        combine(
                serviceVm.uiState
                    // Wait for the first connected state
                    .dropWhile { it !is ServiceState.Connected },
                navController.appCurrentDestinationFlow,
            ) { serviceState, destination ->
                val backstackContainsNoDaemon =
                    navController.backStackContains(NoDaemonScreenDestination)
                // If we are have NoDaemonScreen on backstack and received a connected state, pop it
                if (backstackContainsNoDaemon && serviceState == ServiceState.Connected) {
                    DaemonNavigation.RemoveNoDaemonScreen
                }
                // If we are not connected to and expect to have a service connection, show
                // NoDaemonScreen.
                else if (
                    backstackContainsNoDaemon ||
                        destination.shouldHaveServiceConnection() &&
                            serviceState == ServiceState.Disconnected
                ) {
                    DaemonNavigation.ShowNoDaemonScreen
                } else {
                    // Else, we don't have noDaemonScreen on backstack and don't do anything
                    null
                }
            }
            // We don't care about null
            .filterNotNull()
            // Only care about changes
            .distinctUntilChanged()
            .collect {
                when (it) {
                    DaemonNavigation.ShowNoDaemonScreen ->
                        navController.navigate(NoDaemonScreenDestination)
                    DaemonNavigation.RemoveNoDaemonScreen ->
                        navController.popBackStack(NoDaemonScreenDestination, true)
                }
            }
    }

    // Globally show the changelog
    val changeLogsViewModel = koinViewModel<ChangelogViewModel>()
    LaunchedEffect(Unit) {
        changeLogsViewModel.uiSideEffect.collect {

            // Wait until we are in an acceptable destination
            navController.currentBackStackEntryFlow
                .map { it.destination() }
                .first { it in changeLogDestinations }

            navController.navigate(ChangelogDestination(it).route)
        }
    }
}

private fun Destination.shouldHaveServiceConnection() = this !in noServiceDestinations

sealed interface DaemonNavigation {
    data object ShowNoDaemonScreen : DaemonNavigation

    data object RemoveNoDaemonScreen : DaemonNavigation
}

fun NavController.backStackContains(destination: Destination) =
    try {
        getBackStackEntry(destination.route)
        true
    } catch (e: IllegalArgumentException) {
        false
    }
