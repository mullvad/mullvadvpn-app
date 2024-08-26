package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.DefaultDnsOptions
import net.mullvad.mullvadvpn.lib.model.Mtu
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.model.PortRange
import net.mullvad.mullvadvpn.lib.model.QuantumResistantState
import net.mullvad.mullvadvpn.lib.model.SelectedObfuscation
import net.mullvad.mullvadvpn.viewmodel.CustomDnsItem

data class VpnSettingsUiState(
    val mtu: Mtu?,
    val isAutoConnectEnabled: Boolean,
    val isLocalNetworkSharingEnabled: Boolean,
    val isDaitaEnabled: Boolean,
    val isCustomDnsEnabled: Boolean,
    val customDnsItems: List<CustomDnsItem>,
    val contentBlockersOptions: DefaultDnsOptions,
    val selectedObfuscation: SelectedObfuscation,
    val selectedObfuscationPort: Constraint<Port>,
    val quantumResistant: QuantumResistantState,
    val selectedWireguardPort: Constraint<Port>,
    val customWireguardPort: Constraint<Port>?,
    val availablePortRanges: List<PortRange>,
    val systemVpnSettingsAvailable: Boolean,
) {
    val selectObfuscationPortEnabled = selectedObfuscation != SelectedObfuscation.Off

    companion object {
        fun createDefault(
            mtu: Mtu? = null,
            isAutoConnectEnabled: Boolean = false,
            isLocalNetworkSharingEnabled: Boolean = false,
            isDaitaEnabled: Boolean = false,
            isCustomDnsEnabled: Boolean = false,
            customDnsItems: List<CustomDnsItem> = emptyList(),
            contentBlockersOptions: DefaultDnsOptions = DefaultDnsOptions(),
            selectedObfuscation: SelectedObfuscation = SelectedObfuscation.Off,
            selectedObfuscationPort: Constraint<Port> = Constraint.Any,
            quantumResistant: QuantumResistantState = QuantumResistantState.Off,
            selectedWireguardPort: Constraint<Port> = Constraint.Any,
            customWireguardPort: Constraint.Only<Port>? = null,
            availablePortRanges: List<PortRange> = emptyList(),
            systemVpnSettingsAvailable: Boolean = false,
        ) =
            VpnSettingsUiState(
                mtu,
                isAutoConnectEnabled,
                isLocalNetworkSharingEnabled,
                isDaitaEnabled,
                isCustomDnsEnabled,
                customDnsItems,
                contentBlockersOptions,
                selectedObfuscation,
                selectedObfuscationPort,
                quantumResistant,
                selectedWireguardPort,
                customWireguardPort,
                availablePortRanges,
                systemVpnSettingsAvailable,
            )
    }
}
