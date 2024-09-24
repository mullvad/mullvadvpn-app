package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.DefaultDnsOptions
import net.mullvad.mullvadvpn.lib.model.Mtu
import net.mullvad.mullvadvpn.lib.model.ObfuscationMode
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.model.PortRange
import net.mullvad.mullvadvpn.lib.model.QuantumResistantState
import net.mullvad.mullvadvpn.viewmodel.CustomDnsItem

data class VpnSettingsUiState(
    val mtu: Mtu?,
    val isAutoConnectEnabled: Boolean,
    val isLocalNetworkSharingEnabled: Boolean,
    val isDaitaEnabled: Boolean,
    val isCustomDnsEnabled: Boolean,
    val customDnsItems: List<CustomDnsItem>,
    val contentBlockersOptions: DefaultDnsOptions,
    val obfuscationMode: ObfuscationMode,
    val selectedUdp2TcpObfuscationPort: Constraint<Port>,
    val selectedShadowsSocksObfuscationPort: Constraint<Port>,
    val quantumResistant: QuantumResistantState,
    val selectedWireguardPort: Constraint<Port>,
    val customWireguardPort: Port?,
    val availablePortRanges: List<PortRange>,
    val systemVpnSettingsAvailable: Boolean,
    val autoStartAndConnectOnBoot: Boolean,
) {
    val isCustomWireguardPort =
        selectedWireguardPort is Constraint.Only &&
            selectedWireguardPort.value == customWireguardPort

    companion object {
        fun createDefault(
            mtu: Mtu? = null,
            isAutoConnectEnabled: Boolean = false,
            isLocalNetworkSharingEnabled: Boolean = false,
            isDaitaEnabled: Boolean = false,
            isCustomDnsEnabled: Boolean = false,
            customDnsItems: List<CustomDnsItem> = emptyList(),
            contentBlockersOptions: DefaultDnsOptions = DefaultDnsOptions(),
            obfuscationMode: ObfuscationMode = ObfuscationMode.Off,
            selectedUdp2TcpObfuscationPort: Constraint<Port> = Constraint.Any,
            selectedShadowsSocksObfuscationPort: Constraint<Port> = Constraint.Any,
            quantumResistant: QuantumResistantState = QuantumResistantState.Off,
            selectedWireguardPort: Constraint<Port> = Constraint.Any,
            customWireguardPort: Port? = null,
            availablePortRanges: List<PortRange> = emptyList(),
            systemVpnSettingsAvailable: Boolean = false,
            autoStartAndConnectOnBoot: Boolean = false,
        ) =
            VpnSettingsUiState(
                mtu,
                isAutoConnectEnabled,
                isLocalNetworkSharingEnabled,
                isDaitaEnabled,
                isCustomDnsEnabled,
                customDnsItems,
                contentBlockersOptions,
                obfuscationMode,
                selectedUdp2TcpObfuscationPort,
                selectedShadowsSocksObfuscationPort,
                quantumResistant,
                selectedWireguardPort,
                customWireguardPort,
                availablePortRanges,
                systemVpnSettingsAvailable,
                autoStartAndConnectOnBoot,
            )
    }
}
