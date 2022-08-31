package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow

class ConnectViewModel : ViewModel() {
    private val _isTunnelInfoExpanded = MutableStateFlow(false)
    val isTunnelInfoExpanded = _isTunnelInfoExpanded.asStateFlow()

    fun toggleTunnelInfoExpansion() {
        _isTunnelInfoExpanded.value = _isTunnelInfoExpanded.value.not()
    }
}
