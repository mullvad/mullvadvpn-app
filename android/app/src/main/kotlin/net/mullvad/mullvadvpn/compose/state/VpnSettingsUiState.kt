package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.DefaultDnsOptions
import net.mullvad.mullvadvpn.lib.model.IpVersion
import net.mullvad.mullvadvpn.lib.model.Mtu
import net.mullvad.mullvadvpn.lib.model.ObfuscationMode
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.model.PortRange
import net.mullvad.mullvadvpn.lib.model.QuantumResistantState
import net.mullvad.mullvadvpn.viewmodel.CustomDnsItem

data class VpnSettingsUiState(
    val mtu: Mtu?,
    val isLocalNetworkSharingEnabled: Boolean,
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
    val deviceIpVersion: Constraint<IpVersion>,
    val isIPv6Enabled: Boolean,
    val routeIpv6Traffic: Boolean,
) {
    val isCustomWireguardPort =
        selectedWireguardPort is Constraint.Only &&
            selectedWireguardPort.value == customWireguardPort

    val isWireguardPortEnabled =
        obfuscationMode == ObfuscationMode.Auto || obfuscationMode == ObfuscationMode.Off

    companion object {
        fun createDefault(
            mtu: Mtu? = null,
            isLocalNetworkSharingEnabled: Boolean = false,
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
            deviceIpVersion: Constraint<IpVersion> = Constraint.Any,
            isIPv6Enabled: Boolean = true,
            routeIpv6Traffic: Boolean = true,
        ) =
            VpnSettingsUiState(
                mtu,
                isLocalNetworkSharingEnabled,
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
                deviceIpVersion,
                isIPv6Enabled,
                routeIpv6Traffic,
            )
    }
}
