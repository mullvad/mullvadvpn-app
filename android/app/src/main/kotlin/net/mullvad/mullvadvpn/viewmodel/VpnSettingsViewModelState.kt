package net.mullvad.mullvadvpn.viewmodel

import net.mullvad.mullvadvpn.compose.state.VpnSettingsUiState
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.DefaultDnsOptions
import net.mullvad.mullvadvpn.lib.model.IpVersion
import net.mullvad.mullvadvpn.lib.model.Mtu
import net.mullvad.mullvadvpn.lib.model.ObfuscationMode
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.model.PortRange
import net.mullvad.mullvadvpn.lib.model.QuantumResistantState

data class VpnSettingsViewModelState(
    val mtuValue: Mtu?,
    val isLocalNetworkSharingEnabled: Boolean,
    val isCustomDnsEnabled: Boolean,
    val customDnsList: List<CustomDnsItem>,
    val contentBlockersOptions: DefaultDnsOptions,
    val obfuscationMode: ObfuscationMode,
    val selectedUdp2TcpObfuscationPort: Constraint<Port>,
    val selectedShadowsocksObfuscationPort: Constraint<Port>,
    val quantumResistant: QuantumResistantState,
    val selectedWireguardPort: Constraint<Port>,
    val customWireguardPort: Port?,
    val availablePortRanges: List<PortRange>,
    val systemVpnSettingsAvailable: Boolean,
    val autoStartAndConnectOnBoot: Boolean,
    val deviceIpVersion: Constraint<IpVersion>,
    val ipv6Enabled: Boolean,
    val routeIpv6: Boolean,
) {
    val isCustomWireguardPort =
        selectedWireguardPort is Constraint.Only &&
            selectedWireguardPort.value == customWireguardPort

    fun toUiState(): VpnSettingsUiState =
        VpnSettingsUiState(
            mtuValue,
            isLocalNetworkSharingEnabled,
            isCustomDnsEnabled,
            customDnsList,
            contentBlockersOptions,
            obfuscationMode,
            selectedUdp2TcpObfuscationPort,
            selectedShadowsocksObfuscationPort,
            quantumResistant,
            selectedWireguardPort,
            customWireguardPort,
            availablePortRanges,
            systemVpnSettingsAvailable,
            autoStartAndConnectOnBoot,
            deviceIpVersion,
            ipv6Enabled,
            routeIpv6,
        )

    companion object {
        fun default() =
            VpnSettingsViewModelState(
                mtuValue = null,
                isLocalNetworkSharingEnabled = false,
                isCustomDnsEnabled = false,
                customDnsList = listOf(),
                contentBlockersOptions = DefaultDnsOptions(),
                obfuscationMode = ObfuscationMode.Auto,
                selectedUdp2TcpObfuscationPort = Constraint.Any,
                selectedShadowsocksObfuscationPort = Constraint.Any,
                quantumResistant = QuantumResistantState.Off,
                selectedWireguardPort = Constraint.Any,
                customWireguardPort = null,
                availablePortRanges = emptyList(),
                systemVpnSettingsAvailable = false,
                autoStartAndConnectOnBoot = false,
                deviceIpVersion = Constraint.Any,
                ipv6Enabled = false,
                routeIpv6 = false,
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
