package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import net.mullvad.mullvadvpn.compose.state.DeviceRevokedUiState
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionContainer
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.talpid.util.callbackFlowFromSubscription

// TODO: Refactor AccountCache and ConnectionProxy and inject those rather than injecting
//  ServiceConnectionManager here.
class DeviceRevokedViewModel(
    private val serviceConnectionManager: ServiceConnectionManager,
    scope: CoroutineScope = CoroutineScope(Dispatchers.IO)
) : ViewModel() {

    val uiState = serviceConnectionManager.connectionState
        .map { connectionState -> connectionState.readyContainer()?.connectionProxy }
        .flatMapLatest { proxy ->
            proxy?.onUiStateChange
                ?.callbackFlowFromSubscription(this)
                ?.map {
                    if (it.isSecured()) {
                        DeviceRevokedUiState.SECURED
                    } else {
                        DeviceRevokedUiState.UNSECURED
                    }
                }
                ?: flowOf(DeviceRevokedUiState.UNKNOWN)
        }
        .stateIn(
            scope,
            SharingStarted.Lazily,
            DeviceRevokedUiState.UNKNOWN
        )

    fun onGoToLoginClicked() {
        serviceContainer()?.let { container ->
            if (container.connectionProxy.state.isSecured()) {
                container.connectionProxy.disconnect()
            }
            container.accountCache.logout()
        }
    }

    private fun serviceContainer(): ServiceConnectionContainer? {
        return serviceConnectionManager.connectionState.value.readyContainer()
    }
}
