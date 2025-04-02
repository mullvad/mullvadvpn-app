package net.mullvad.mullvadvpn.viewmodel

import net.mullvad.mullvadvpn.compose.state.VpnSettingItem
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.DefaultDnsOptions
import net.mullvad.mullvadvpn.lib.model.IpVersion
import net.mullvad.mullvadvpn.lib.model.Mtu
import net.mullvad.mullvadvpn.lib.model.ObfuscationMode
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.model.PortRange
import net.mullvad.mullvadvpn.lib.model.QuantumResistantState

sealed interface VpnSettingsUiState {
    data object Loading : VpnSettingsUiState
    data class Content(val settings: List<VpnSettingItem>): VpnSettingsUiState
}

data class VpnSettingsViewModelState(
    val mtu: Mtu?,
    val isLocalNetworkSharingEnabled: Boolean,
    val isCustomDnsEnabled: Boolean,
    val customDnsItems: List<CustomDnsItem>,
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
    val isIpv6Enabled: Boolean,
    val isContentBlockersExpanded: Boolean
) {
    val isCustomWireguardPort =
        selectedWireguardPort is Constraint.Only &&
            selectedWireguardPort.value == customWireguardPort

    companion object {
        fun default() =
            VpnSettingsViewModelState(
                mtu = null,
                isLocalNetworkSharingEnabled = false,
                isCustomDnsEnabled = false,
                customDnsItems = listOf(),
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
                isIpv6Enabled = false,
                isContentBlockersExpanded = false
            )
    }
}

data class CustomDnsItem(val address: String, val isLocal: Boolean, val isIpv6: Boolean) {
    companion object {
        private const val EMPTY_STRING = ""

        fun default(): CustomDnsItem {
            return CustomDnsItem(address = EMPTY_STRING, isLocal = false, isIpv6 = false)
        }
    }
}
