package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.shareIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.lib.common.constant.KEY_REQUEST_VPN_PERMISSION
import net.mullvad.mullvadvpn.lib.intent.IntentProvider
import net.mullvad.mullvadvpn.ui.serviceconnection.ConnectionProxy

class VpnPermissionViewModel(
    intentProvider: IntentProvider,
    private val connectionProxy: ConnectionProxy
) : ViewModel() {
    val uiSideEffect: Flow<VpnPermissionSideEffect> =
        intentProvider.intents
            .filter { it?.action == KEY_REQUEST_VPN_PERMISSION }
            .distinctUntilChanged()
            .map { VpnPermissionSideEffect.ShowDialog }
            .shareIn(viewModelScope, SharingStarted.WhileSubscribed())

    fun connect() {
        viewModelScope.launch { connectionProxy.connect(ignorePermission = true) }
    }
}

sealed interface VpnPermissionSideEffect {
    data object ShowDialog : VpnPermissionSideEffect
}
