package net.mullvad.mullvadvpn.viewmodel

import android.content.res.Resources
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import arrow.core.Either
import arrow.core.right
import co.touchlab.kermit.Logger
import com.ramcosta.composedestinations.generated.destinations.SelectPortDestination
import com.ramcosta.composedestinations.spec.Direction
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.WhileSubscribed
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.state.SelectPortUiState
import net.mullvad.mullvadvpn.constant.SHADOWSOCKS_AVAILABLE_PORTS
import net.mullvad.mullvadvpn.constant.SHADOWSOCKS_PRESET_PORTS
import net.mullvad.mullvadvpn.constant.UDP2TCP_PRESET_PORTS
import net.mullvad.mullvadvpn.constant.VIEW_MODEL_STOP_TIMEOUT
import net.mullvad.mullvadvpn.constant.WIREGUARD_PRESET_PORTS
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.ObfuscationSettings
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.model.PortRange
import net.mullvad.mullvadvpn.lib.model.PortType
import net.mullvad.mullvadvpn.lib.model.SetObfuscationOptionsError
import net.mullvad.mullvadvpn.repository.RelayListRepository
import net.mullvad.mullvadvpn.repository.SettingsRepository
import net.mullvad.mullvadvpn.util.Lc
import net.mullvad.mullvadvpn.util.toLc

class SelectPortViewModel(
    private val settingsRepository: SettingsRepository,
    private val resources: Resources,
    relayListRepository: RelayListRepository,
    savedStateHandle: SavedStateHandle,
) : ViewModel() {

    private val navArgs = SelectPortDestination.argsFrom(savedStateHandle)
    private val portType = navArgs.portType

    private val initialOrCustomPort = MutableStateFlow<Port?>(null)

    init {
        viewModelScope.launch {
            val initialSettings = settingsRepository.settingsUpdates.filterNotNull().first()
            initialOrCustomPort.value =
                initialSettings.obfuscationSettings.port(portType).getOrNull()
        }
    }

    val uiState: StateFlow<Lc<Unit, SelectPortUiState>> =
        combine(
                settingsRepository.settingsUpdates.filterNotNull(),
                relayListRepository.portRanges,
                initialOrCustomPort,
            ) { settings, wireguardPortRanges, initialOrCustomPort ->
                val portTypeState = portType.uiState(wireguardPortRanges = wireguardPortRanges)
                val customPort =
                    if (initialOrCustomPort !in portTypeState.presetPorts) initialOrCustomPort
                    else null

                SelectPortUiState(
                        portType = portType,
                        port = settings.obfuscationSettings.port(portType),
                        presetPorts = portTypeState.presetPorts,
                        customPortEnabled = portTypeState.customPortEnabled,
                        title = portTypeState.title,
                        allowedPortRanges = portTypeState.allowedPortRanges,
                        customPort = customPort,
                    )
                    .toLc<Unit, SelectPortUiState>()
            }
            .stateIn(
                scope = viewModelScope,
                started = SharingStarted.WhileSubscribed(VIEW_MODEL_STOP_TIMEOUT),
                initialValue = Lc.Loading(Unit),
            )

    fun onPortSelected(port: Constraint<Port>) {
        viewModelScope.launch {
            updatePort(port)
                .onLeft { Logger.e("Select shadowsocks port error $it") }
                .onRight {
                    val presets = uiState.value.contentOrNull()?.presetPorts ?: emptyList()
                    if (port is Constraint.Only && port.value !in presets) {
                        initialOrCustomPort.update { port.getOrNull() }
                    }
                }
        }
    }

    private suspend fun updatePort(
        port: Constraint<Port>
    ): Either<SetObfuscationOptionsError, Unit> =
        when (portType) {
            PortType.Udp2Tcp -> settingsRepository.setCustomUdp2TcpObfuscationPort(port)
            PortType.Shadowsocks -> settingsRepository.setCustomShadowsocksObfuscationPort(port)
            PortType.Wireguard -> settingsRepository.setCustomWireguardPort(port)
            PortType.Lwo -> Unit.right()
        }

    fun resetCustomPort() {
        val isCustom = uiState.value.contentOrNull()?.isCustom == true
        initialOrCustomPort.update { null }
        // If custom port was selected, update selection to be any.
        if (isCustom) {
            viewModelScope.launch { updatePort(Constraint.Any) }
        }
    }

    private fun PortType.uiState(wireguardPortRanges: List<PortRange>): PortTypeUiState =
        when (this) {
            PortType.Udp2Tcp ->
                PortTypeUiState(
                    presetPorts = UDP2TCP_PRESET_PORTS,
                    allowedPortRanges = emptyList(),
                    customPortEnabled = false,
                    title = resources.getString(R.string.udp_over_tcp),
                )
            PortType.Shadowsocks ->
                PortTypeUiState(
                    presetPorts = SHADOWSOCKS_PRESET_PORTS,
                    allowedPortRanges = SHADOWSOCKS_AVAILABLE_PORTS,
                    customPortEnabled = true,
                    title = resources.getString(R.string.shadowsocks),
                )
            PortType.Wireguard ->
                PortTypeUiState(
                    presetPorts = WIREGUARD_PRESET_PORTS,
                    allowedPortRanges = wireguardPortRanges,
                    customPortEnabled = true,
                    title = resources.getString(R.string.wireguard_port_title),
                )
            PortType.Lwo ->
                PortTypeUiState(
                    presetPorts = emptyList(),
                    allowedPortRanges = emptyList(),
                    customPortEnabled = false,
                    title = resources.getString(R.string.lwo),
                )
        }

    private fun ObfuscationSettings.port(portType: PortType): Constraint<Port> =
        when (portType) {
            PortType.Udp2Tcp -> udp2tcp.port
            PortType.Shadowsocks -> shadowsocks.port
            PortType.Wireguard -> wireguardPort
            PortType.Lwo -> Constraint.Any
        }
}

data class PortTypeUiState(
    val presetPorts: List<Port>,
    val allowedPortRanges: List<PortRange>,
    val customPortEnabled: Boolean,
    val title: String,
    val infoDestination: Direction? = null,
)
