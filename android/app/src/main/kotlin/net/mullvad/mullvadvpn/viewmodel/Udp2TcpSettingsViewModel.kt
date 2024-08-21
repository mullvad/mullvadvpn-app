package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import co.touchlab.kermit.Logger
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.state.Udp2TcpSettingsState
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.repository.SettingsRepository

class Udp2TcpSettingsViewModel(private val repository: SettingsRepository) : ViewModel() {
    val uiState: StateFlow<Udp2TcpSettingsState> =
        repository.settingsUpdates
            .filterNotNull()
            .map { settings ->
                Udp2TcpSettingsState(port = settings.obfuscationSettings.udp2tcp.port)
            }
            .stateIn(
                scope = viewModelScope,
                started = SharingStarted.WhileSubscribed(),
                initialValue = Udp2TcpSettingsState(),
            )

    fun onObfuscationPortSelected(port: Constraint<Port>) {
        viewModelScope.launch {
            repository.setCustomUdp2TcpObfuscationPort(port).onLeft {
                Logger.e("Select udp to tcp port error $it")
            }
        }
    }
}
