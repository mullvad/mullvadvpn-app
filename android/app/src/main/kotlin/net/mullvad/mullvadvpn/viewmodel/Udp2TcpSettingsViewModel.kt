package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import co.touchlab.kermit.Logger
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.WhileSubscribed
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.state.Udp2TcpSettingsUiState
import net.mullvad.mullvadvpn.constant.VIEW_MODEL_STOP_TIMEOUT
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.repository.SettingsRepository
import net.mullvad.mullvadvpn.util.Lc
import net.mullvad.mullvadvpn.util.toLc

class Udp2TcpSettingsViewModel(private val repository: SettingsRepository) : ViewModel() {
    val uiState: StateFlow<Lc<Unit, Udp2TcpSettingsUiState>> =
        repository.settingsUpdates
            .filterNotNull()
            .map { settings ->
                Udp2TcpSettingsUiState(port = settings.obfuscationSettings.udp2tcp.port)
                    .toLc<Unit, Udp2TcpSettingsUiState>()
            }
            .stateIn(
                scope = viewModelScope,
                started = SharingStarted.WhileSubscribed(VIEW_MODEL_STOP_TIMEOUT),
                initialValue = Lc.Loading(Unit),
            )

    fun onObfuscationPortSelected(port: Constraint<Port>) {
        viewModelScope.launch {
            repository.setCustomUdp2TcpObfuscationPort(port).onLeft {
                Logger.e("Select udp to tcp port error $it")
            }
        }
    }
}
