package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionState

class ServiceConnectionViewModel(serviceConnectionManager: ServiceConnectionManager) : ViewModel() {
    val uiState =
        serviceConnectionManager.connectionState
            .map {
                when (it) {
                    is ServiceConnectionState.ConnectedNotReady -> ServiceState.Disconnected
                    is ServiceConnectionState.ConnectedReady -> ServiceState.Connected
                    ServiceConnectionState.Disconnected -> ServiceState.Disconnected
                }
            }
            .stateIn(
                scope = viewModelScope,
                started = SharingStarted.Eagerly,
                initialValue = ServiceState.Disconnected
            )
}

sealed interface ServiceState {
    data object Disconnected : ServiceState

    data object Connected : ServiceState
}
