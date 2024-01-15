package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.model.Constraint
import net.mullvad.mullvadvpn.model.DefaultDnsOptions
import net.mullvad.mullvadvpn.model.Port
import net.mullvad.mullvadvpn.model.PortRange
import net.mullvad.mullvadvpn.model.QuantumResistantState
import net.mullvad.mullvadvpn.model.SelectedObfuscation
import net.mullvad.mullvadvpn.viewmodel.CustomDnsItem

data class VpnSettingsUiState(
    val mtu: String,
    val isAutoConnectEnabled: Boolean,
    val isLocalNetworkSharingEnabled: Boolean,
    val isCustomDnsEnabled: Boolean,
    val customDnsItems: List<CustomDnsItem>,
    val contentBlockersOptions: DefaultDnsOptions,
    val selectedObfuscation: SelectedObfuscation,
    val quantumResistant: QuantumResistantState,
    val selectedWireguardPort: Constraint<Port>,
    val customWireguardPort: Constraint<Port>?,
    val availablePortRanges: List<PortRange>,
    val systemVpnSettingsAvailable: Boolean,
) {

    companion object {
        fun createDefault(
            mtu: String = "",
            isAutoConnectEnabled: Boolean = false,
            isLocalNetworkSharingEnabled: Boolean = false,
            isCustomDnsEnabled: Boolean = false,
            customDnsItems: List<CustomDnsItem> = emptyList(),
            contentBlockersOptions: DefaultDnsOptions = DefaultDnsOptions(),
            selectedObfuscation: SelectedObfuscation = SelectedObfuscation.Off,
            quantumResistant: QuantumResistantState = QuantumResistantState.Off,
            selectedWireguardPort: Constraint<Port> = Constraint.Any(),
            customWireguardPort: Constraint.Only<Port>? = null,
            availablePortRanges: List<PortRange> = emptyList(),
            systemVpnSettingsAvailable: Boolean = false,
        ) =
            VpnSettingsUiState(
                mtu,
                isAutoConnectEnabled,
                isLocalNetworkSharingEnabled,
                isCustomDnsEnabled,
                customDnsItems,
                contentBlockersOptions,
                selectedObfuscation,
                quantumResistant,
                selectedWireguardPort,
                customWireguardPort,
                availablePortRanges,
                systemVpnSettingsAvailable
            )
    }
}
