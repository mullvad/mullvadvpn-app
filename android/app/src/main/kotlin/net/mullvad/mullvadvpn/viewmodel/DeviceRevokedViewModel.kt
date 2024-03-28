package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.state.DeviceRevokedUiState
import net.mullvad.mullvadvpn.repository.AccountRepository
import net.mullvad.mullvadvpn.ui.serviceconnection.ConnectionProxy
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager

// TODO: Refactor ConnectionProxy to be easily injectable rather than injecting
//  ServiceConnectionManager here.
class DeviceRevokedViewModel(
    private val serviceConnectionManager: ServiceConnectionManager,
    private val accountRepository: AccountRepository,
    private val connectionProxy: ConnectionProxy,
    dispatcher: CoroutineDispatcher = Dispatchers.IO
) : ViewModel() {

    val uiState =
        serviceConnectionManager.connectionState
            .map { connectionState -> connectionProxy }
            .flatMapLatest { proxy ->
                proxy.tunnelState.map {
                    if (it.isSecured()) {
                        DeviceRevokedUiState.SECURED
                    } else {
                        DeviceRevokedUiState.UNSECURED
                    }
                } ?: flowOf(DeviceRevokedUiState.UNKNOWN)
            }
            .stateIn(
                scope = CoroutineScope(dispatcher),
                started = SharingStarted.WhileSubscribed(),
                initialValue = DeviceRevokedUiState.UNKNOWN
            )

    private val _uiSideEffect = Channel<DeviceRevokedSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    fun onGoToLoginClicked() {
        /*serviceConnectionManager.connectionProxy()?.let { proxy ->
            if (proxy.state.isSecured()) {
                proxy.disconnect()
            }
        }*/

        viewModelScope.launch {
            connectionProxy.disconnect()
            accountRepository.logout()
        }

        viewModelScope.launch { _uiSideEffect.send(DeviceRevokedSideEffect.NavigateToLogin) }
    }
}

sealed interface DeviceRevokedSideEffect {
    data object NavigateToLogin : DeviceRevokedSideEffect
}
