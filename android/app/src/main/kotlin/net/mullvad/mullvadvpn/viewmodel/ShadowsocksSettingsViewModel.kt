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
import net.mullvad.mullvadvpn.compose.state.ShadowsocksSettingsState
import net.mullvad.mullvadvpn.constant.SHADOWSOCKS_PRESET_PORTS
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.model.Settings
import net.mullvad.mullvadvpn.repository.RelayListRepository
import net.mullvad.mullvadvpn.repository.SettingsRepository

class ShadowsocksSettingsViewModel(
    private val settingsRepository: SettingsRepository,
    relayListRepository: RelayListRepository
) : ViewModel() {

    private val customPort = MutableStateFlow<Port?>(null)

    val uiState: StateFlow<ShadowsocksSettingsState> =
        combine(
                settingsRepository.settingsUpdates.filterNotNull(),
                customPort,
                relayListRepository.shadowsocksPortRanges
            ) { settings, customPort, portRanges ->
                ShadowsocksSettingsState(
                    port = settings.getShadowSocksPort(),
                    customPort = customPort,
                    validPortRanges = portRanges
                )
            }
            .stateIn(
                scope = viewModelScope,
                started = SharingStarted.WhileSubscribed(),
                initialValue = ShadowsocksSettingsState()
            )

    init {
        viewModelScope.launch {
            val initialSettings = settingsRepository.settingsUpdates.filterNotNull().first()
            customPort.update {
                val initialPort = initialSettings.getShadowSocksPort()
                if (SHADOWSOCKS_PRESET_PORTS.contains(initialPort.getOrNull()).not()) {
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
                    if (
                        port is Constraint.Only &&
                            SHADOWSOCKS_PRESET_PORTS.contains(port.value).not()
                    ) {
                        customPort.update { port.getOrNull() }
                    }
                }
        }
    }

    fun resetCustomPort() {
        val isCustom = uiState.value.isCustom
        customPort.update { null }
        // If custom port was selected, update selection to be any.
        if (isCustom) {
            viewModelScope.launch {
                settingsRepository.setCustomShadowsocksObfuscationPort(Constraint.Any)
            }
        }
    }

    private fun Settings.getShadowSocksPort() = obfuscationSettings.shadowsocks.port
}
