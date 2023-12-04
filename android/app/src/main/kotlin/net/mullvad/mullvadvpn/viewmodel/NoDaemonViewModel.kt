package net.mullvad.mullvadvpn.viewmodel

import android.os.Bundle
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.LifecycleEventObserver
import androidx.lifecycle.LifecycleOwner
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import androidx.navigation.NavController
import androidx.navigation.NavDestination
import com.ramcosta.composedestinations.spec.DestinationSpec
import com.ramcosta.composedestinations.utils.destination
import kotlin.time.Duration.Companion.seconds
import kotlinx.coroutines.FlowPreview
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.debounce
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.shareIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.destinations.PrivacyDisclaimerDestination
import net.mullvad.mullvadvpn.compose.destinations.SplashDestination
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionState

private val noServiceDestinations = listOf(SplashDestination, PrivacyDisclaimerDestination)

class NoDaemonViewModel(serviceConnectionManager: ServiceConnectionManager) :
    ViewModel(), LifecycleEventObserver, NavController.OnDestinationChangedListener {

    private val lifecycleFlow: MutableSharedFlow<Lifecycle.Event> = MutableSharedFlow()
    private val destinationFlow: MutableSharedFlow<DestinationSpec<*>> = MutableSharedFlow()

    @OptIn(FlowPreview::class)
    val uiSideEffect =
        combine(lifecycleFlow, serviceConnectionManager.connectionState, destinationFlow) {
                event,
                connEvent,
                destination ->
                toDaemonState(event, connEvent, destination)
            }
            .map { state ->
                when (state) {
                    is DaemonState.Show -> DaemonScreenEvent.Show
                    is DaemonState.Hidden.Ignored -> DaemonScreenEvent.Remove
                    DaemonState.Hidden.Connected -> DaemonScreenEvent.Remove
                }
            }
            .distinctUntilChanged()
            // We debounce any disconnected state to let the UI have some time to connect after a
            // onStart/onStop event.
            .debounce {
                when (it) {
                    is DaemonScreenEvent.Remove -> 0.seconds
                    is DaemonScreenEvent.Show -> SERVICE_DISCONNECT_DEBOUNCE
                }
            }
            .distinctUntilChanged()
            .shareIn(viewModelScope, SharingStarted.Eagerly)

    override fun onStateChanged(source: LifecycleOwner, event: Lifecycle.Event) {
        viewModelScope.launch { lifecycleFlow.emit(event) }
    }

    private fun toDaemonState(
        lifecycleEvent: Lifecycle.Event,
        serviceState: ServiceConnectionState,
        currentDestination: DestinationSpec<*>
    ): DaemonState {
        // In these destinations we don't care about showing the NoDaemonScreen
        if (currentDestination in noServiceDestinations) {
            return DaemonState.Hidden.Ignored
        }

        return if (lifecycleEvent.targetState.isAtLeast(Lifecycle.State.STARTED)) {
            // If we are started we want to show the overlay if we are not connected to daemon
            when (serviceState) {
                is ServiceConnectionState.ConnectedNotReady,
                ServiceConnectionState.Disconnected -> DaemonState.Show
                is ServiceConnectionState.ConnectedReady -> DaemonState.Hidden.Connected
            }
        } else {
            // If we are stopped we intentionally stop service and don't care about showing overlay.
            DaemonState.Hidden.Ignored
        }
    }

    override fun onDestinationChanged(
        controller: NavController,
        destination: NavDestination,
        arguments: Bundle?
    ) {
        viewModelScope.launch {
            controller.currentBackStackEntry?.destination()?.let { destinationFlow.emit(it) }
        }
    }

    companion object {
        private val SERVICE_DISCONNECT_DEBOUNCE = 1.seconds
    }
}

sealed interface DaemonState {
    data object Show : DaemonState

    sealed interface Hidden : DaemonState {
        data object Ignored : Hidden

        data object Connected : Hidden
    }
}

sealed interface DaemonScreenEvent {
    data object Show : DaemonScreenEvent

    data object Remove : DaemonScreenEvent
}
