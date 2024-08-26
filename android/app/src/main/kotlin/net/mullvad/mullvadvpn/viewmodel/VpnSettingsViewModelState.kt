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
    val isDaitaEnabled: Boolean,
    val isCustomDnsEnabled: Boolean,
    val customDnsList: List<CustomDnsItem>,
    val contentBlockersOptions: DefaultDnsOptions,
    val selectedObfuscation: SelectedObfuscation,
    val selectedObfuscationPort: Constraint<Port>,
    val quantumResistant: QuantumResistantState,
    val selectedWireguardPort: Constraint<Port>,
    val customWireguardPort: Constraint<Port>?,
    val availablePortRanges: List<PortRange>,
    val systemVpnSettingsAvailable: Boolean,
) {
    fun toUiState(): VpnSettingsUiState =
        VpnSettingsUiState(
            mtuValue,
            isAutoConnectEnabled,
            isLocalNetworkSharingEnabled,
            isDaitaEnabled,
            isCustomDnsEnabled,
            customDnsList,
            contentBlockersOptions,
            selectedObfuscation,
            selectedObfuscationPort,
            quantumResistant,
            selectedWireguardPort,
            customWireguardPort,
            availablePortRanges,
            systemVpnSettingsAvailable,
        )

    companion object {
        fun default() =
            VpnSettingsViewModelState(
                mtuValue = null,
                isAutoConnectEnabled = false,
                isLocalNetworkSharingEnabled = false,
                isDaitaEnabled = false,
                isCustomDnsEnabled = false,
                customDnsList = listOf(),
                contentBlockersOptions = DefaultDnsOptions(),
                selectedObfuscation = SelectedObfuscation.Auto,
                selectedObfuscationPort = Constraint.Any,
                quantumResistant = QuantumResistantState.Off,
                selectedWireguardPort = Constraint.Any,
                customWireguardPort = null,
                availablePortRanges = emptyList(),
                systemVpnSettingsAvailable = false,
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
