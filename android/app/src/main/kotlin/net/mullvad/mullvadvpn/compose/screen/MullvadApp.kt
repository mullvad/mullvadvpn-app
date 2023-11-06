package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.semantics.testTagsAsResourceId
import androidx.navigation.NavHostController
import com.ramcosta.composedestinations.DestinationsNavHost
import com.ramcosta.composedestinations.rememberNavHostEngine
import com.ramcosta.composedestinations.utils.destination
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.map
import net.mullvad.mullvadvpn.compose.NavGraphs
import net.mullvad.mullvadvpn.compose.destinations.ChangelogDestination
import net.mullvad.mullvadvpn.compose.destinations.ConnectDestination
import net.mullvad.mullvadvpn.compose.destinations.OutOfTimeDestination
import net.mullvad.mullvadvpn.viewmodel.ChangelogViewModel
import org.koin.androidx.compose.koinViewModel

private val changeLogDestinations = listOf(ConnectDestination, OutOfTimeDestination)

@OptIn(ExperimentalComposeUiApi::class)
@Composable
fun MullvadApp() {
    val engine = rememberNavHostEngine()
    val navController: NavHostController = engine.rememberNavController()

    DestinationsNavHost(
        modifier = Modifier.semantics { testTagsAsResourceId = true }.fillMaxSize(),
        engine = engine,
        navController = navController,
        navGraph = NavGraphs.root
    )

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
