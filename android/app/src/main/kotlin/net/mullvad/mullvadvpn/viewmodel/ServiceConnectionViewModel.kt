package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.FlowPreview
import kotlin.time.Duration.Companion.seconds
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.debounce
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionState

class ServiceConnectionViewModel(serviceConnectionManager: ServiceConnectionManager) : ViewModel() {
    @OptIn(FlowPreview::class)
    val uiState =
        serviceConnectionManager.connectionState
            .map {
                when (it) {
                    is ServiceConnectionState.ConnectedNotReady -> ServiceState.Disconnected
                    is ServiceConnectionState.ConnectedReady -> ServiceState.Connected
                    ServiceConnectionState.Disconnected -> ServiceState.Disconnected
                }
            }
            // We debounce any disconnected state to let the UI have some time to connect after a
            // onPaused/onResumed event.
            .debounce {
                when (it) {
                    is ServiceState.Connected -> 0.seconds
                    is ServiceState.Disconnected -> SERVICE_DISCONNECT_DEBOUNCE
                }
            }
            .stateIn(
                scope = viewModelScope,
                started = SharingStarted.Eagerly,
                initialValue = ServiceState.Disconnected
            )


    companion object {
        private val SERVICE_DISCONNECT_DEBOUNCE = 1.seconds
    }
}

sealed interface ServiceState {
    data object Disconnected : ServiceState

    data object Connected : ServiceState
}
