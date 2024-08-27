package net.mullvad.mullvadvpn.viewmodel

import net.mullvad.mullvadvpn.compose.state.VpnSettingsUiState
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.DefaultDnsOptions
import net.mullvad.mullvadvpn.lib.model.Mtu
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.model.PortRange
import net.mullvad.mullvadvpn.lib.model.QuantumResistantState
import net.mullvad.mullvadvpn.lib.model.SelectedObfuscation

data class VpnSettingsViewModelState(
    val mtuValue: Mtu?,
    val isAutoConnectEnabled: Boolean,
    val isLocalNetworkSharingEnabled: Boolean,
    val isCustomDnsEnabled: Boolean,
    val customDnsList: List<CustomDnsItem>,
    val contentBlockersOptions: DefaultDnsOptions,
    val selectedObfuscation: SelectedObfuscation,
    val selectedUdp2TcpObfuscationPort: Constraint<Port>,
    val selectedShadowsocksObfuscationPort: Constraint<Port>,
    val quantumResistant: QuantumResistantState,
    val selectedWireguardPort: Constraint<Port>,
    val customWireguardPort: Port?,
    val availablePortRanges: List<PortRange>,
    val systemVpnSettingsAvailable: Boolean,
) {
    val isCustomWireguardPort =
        selectedWireguardPort is Constraint.Only &&
            selectedWireguardPort.value == customWireguardPort

    fun toUiState(): VpnSettingsUiState =
        VpnSettingsUiState(
            mtuValue,
            isAutoConnectEnabled,
            isLocalNetworkSharingEnabled,
            isCustomDnsEnabled,
            customDnsList,
            contentBlockersOptions,
            selectedObfuscation,
            selectedUdp2TcpObfuscationPort,
            selectedShadowsocksObfuscationPort,
            quantumResistant,
            selectedWireguardPort,
            customWireguardPort,
            availablePortRanges,
            systemVpnSettingsAvailable
        )

    companion object {
        fun default() =
            VpnSettingsViewModelState(
                mtuValue = null,
                isAutoConnectEnabled = false,
                isLocalNetworkSharingEnabled = false,
                isCustomDnsEnabled = false,
                customDnsList = listOf(),
                contentBlockersOptions = DefaultDnsOptions(),
                selectedObfuscation = SelectedObfuscation.Auto,
                selectedUdp2TcpObfuscationPort = Constraint.Any,
                selectedShadowsocksObfuscationPort = Constraint.Any,
                quantumResistant = QuantumResistantState.Off,
                selectedWireguardPort = Constraint.Any,
                customWireguardPort = null,
                availablePortRanges = emptyList(),
                systemVpnSettingsAvailable = false
            )
    }
}

data class CustomDnsItem(val address: String, val isLocal: Boolean) {
    companion object {
        private const val EMPTY_STRING = ""

        fun default(): CustomDnsItem {
            return CustomDnsItem(address = EMPTY_STRING, isLocal = false)
        }
    }
}
