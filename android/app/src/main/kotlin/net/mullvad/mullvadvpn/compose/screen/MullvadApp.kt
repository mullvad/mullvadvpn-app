package net.mullvad.mullvadvpn.compose.screen

import android.content.Intent
import androidx.activity.ComponentActivity
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
import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.flow.callbackFlow
import kotlinx.coroutines.flow.filter
import net.mullvad.mullvadvpn.compose.util.CreateVpnProfile
import net.mullvad.mullvadvpn.lib.common.constant.KEY_REQUEST_VPN_PROFILE
import net.mullvad.mullvadvpn.lib.common.util.prepareVpnSafe
import net.mullvad.mullvadvpn.lib.model.PrepareError
import net.mullvad.mullvadvpn.lib.model.Prepared
import net.mullvad.mullvadvpn.util.getActivity
import net.mullvad.mullvadvpn.viewmodel.DaemonScreenEvent
import net.mullvad.mullvadvpn.viewmodel.MullvadAppViewModel
import org.koin.androidx.compose.koinViewModel

@OptIn(ExperimentalComposeUiApi::class)
@Composable
fun MullvadApp() {
    val engine = rememberNavHostEngine()
    val navHostController: NavHostController = engine.rememberNavController()
    val navigator: DestinationsNavigator = navHostController.rememberDestinationsNavigator()

    val mullvadAppViewModel = koinViewModel<MullvadAppViewModel>()

    DisposableEffect(Unit) {
        navHostController.addOnDestinationChangedListener(mullvadAppViewModel)
        onDispose { navHostController.removeOnDestinationChangedListener(mullvadAppViewModel) }
    }

    // Get intents
    val launchVpnPermission =
        rememberLauncherForActivityResult(CreateVpnProfile()) { _ -> mullvadAppViewModel.connect() }
    val activity = LocalContext.current.getActivity() as ComponentActivity
    LaunchedEffect(navHostController) {
        activity
            .intents()
            .filter { it.action == KEY_REQUEST_VPN_PROFILE }
            .collect {
                val prepareResult = activity.prepareVpnSafe().merge()
                when (prepareResult) {
                    is PrepareError.NotPrepared ->
                        launchVpnPermission.launch(prepareResult.prepareIntent)
                    // If legacy or other always on connect at let daemon generate a error state
                    is PrepareError.OtherLegacyAlwaysOnVpn,
                    is PrepareError.OtherAlwaysOnApp,
                    Prepared -> mullvadAppViewModel.connect()
                }
            }
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

private fun ComponentActivity.intents() =
    callbackFlow<Intent> {
        send(intent)

        val listener: (Intent) -> Unit = { trySend(it) }

        addOnNewIntentListener(listener)

        awaitClose { removeOnNewIntentListener(listener) }
    }
