package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import co.touchlab.kermit.Logger
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.state.ShadowsocksSettingsUiState
import net.mullvad.mullvadvpn.constant.SHADOWSOCKS_PRESET_PORTS
import net.mullvad.mullvadvpn.constant.VIEW_MODEL_STOP_TIMEOUT_MS
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.repository.SettingsRepository
import net.mullvad.mullvadvpn.util.Lc
import net.mullvad.mullvadvpn.util.shadowSocksPort
import net.mullvad.mullvadvpn.util.toLc

class ShadowsocksSettingsViewModel(private val settingsRepository: SettingsRepository) :
    ViewModel() {

    private val customPort = MutableStateFlow<Port?>(null)

    val uiState: StateFlow<Lc<Unit, ShadowsocksSettingsUiState>> =
        combine(settingsRepository.settingsUpdates.filterNotNull(), customPort) {
                settings,
                customPort ->
                ShadowsocksSettingsUiState(
                        port = settings.shadowSocksPort(),
                        customPort = customPort,
                    )
                    .toLc<Unit, ShadowsocksSettingsUiState>()
            }
            .stateIn(
                scope = viewModelScope,
                started = SharingStarted.WhileSubscribed(VIEW_MODEL_STOP_TIMEOUT_MS),
                initialValue = Lc.Loading(Unit),
            )

    init {
        viewModelScope.launch {
            val initialSettings = settingsRepository.settingsUpdates.filterNotNull().first()
            customPort.update {
                val initialPort = initialSettings.shadowSocksPort()
                if (initialPort.getOrNull() !in SHADOWSOCKS_PRESET_PORTS) {
                    initialPort.getOrNull()
                } else {
                    null
                }
            }
        }
    }

    fun onObfuscationPortSelected(port: Constraint<Port>) {
        viewModelScope.launch {
            settingsRepository
                .setCustomShadowsocksObfuscationPort(port)
                .onLeft { Logger.e("Select shadowsocks port error $it") }
                .onRight {
                    if (port is Constraint.Only && port.value !in SHADOWSOCKS_PRESET_PORTS) {
                        customPort.update { port.getOrNull() }
                    }
                }
        }
    }

    fun resetCustomPort() {
        val isCustom = uiState.value.contentOrNull()?.isCustom == true
        customPort.update { null }
        // If custom port was selected, update selection to be any.
        if (isCustom) {
            viewModelScope.launch {
                settingsRepository.setCustomShadowsocksObfuscationPort(Constraint.Any)
            }
        }
    }
}
