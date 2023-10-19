package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.model.Constraint
import net.mullvad.mullvadvpn.model.DefaultDnsOptions
import net.mullvad.mullvadvpn.model.Port
import net.mullvad.mullvadvpn.model.PortRange
import net.mullvad.mullvadvpn.model.QuantumResistantState
import net.mullvad.mullvadvpn.model.SelectedObfuscation
import net.mullvad.mullvadvpn.viewmodel.CustomDnsItem
import net.mullvad.mullvadvpn.viewmodel.StagedDns

data class VpnSettingsUiState(
    val mtu: String,
    val isAutoConnectEnabled: Boolean,
    val isLocalNetworkSharingEnabled: Boolean,
    val isCustomDnsEnabled: Boolean,
    val customDnsItems: List<CustomDnsItem>,
    val contentBlockersOptions: DefaultDnsOptions,
    val isAllowLanEnabled: Boolean,
    val selectedObfuscation: SelectedObfuscation,
    val quantumResistant: QuantumResistantState,
    val selectedWireguardPort: Constraint<Port>,
    val availablePortRanges: List<PortRange>,
    val dialog: VpnSettingsDialog?
) {

    companion object {
        fun createDefault(
            mtu: String = "",
            isAutoConnectEnabled: Boolean = false,
            isLocalNetworkSharingEnabled: Boolean = false,
            isCustomDnsEnabled: Boolean = false,
            customDnsItems: List<CustomDnsItem> = emptyList(),
            contentBlockersOptions: DefaultDnsOptions = DefaultDnsOptions(),
            isAllowLanEnabled: Boolean = false,
            selectedObfuscation: SelectedObfuscation = SelectedObfuscation.Off,
            quantumResistant: QuantumResistantState = QuantumResistantState.Off,
            selectedWireguardPort: Constraint<Port> = Constraint.Any(),
            availablePortRanges: List<PortRange> = emptyList(),
            dialog: VpnSettingsDialog? = null
        ) =
            VpnSettingsUiState(
                mtu,
                isAutoConnectEnabled,
                isLocalNetworkSharingEnabled,
                isCustomDnsEnabled,
                customDnsItems,
                contentBlockersOptions,
                isAllowLanEnabled,
                selectedObfuscation,
                quantumResistant,
                selectedWireguardPort,
                availablePortRanges,
                dialog
            )
    }
}

interface VpnSettingsDialog {
    data class Mtu(val mtuEditValue: String) : VpnSettingsDialog

    data class Dns(val stagedDns: StagedDns) : VpnSettingsDialog

    data object LocalNetworkSharingInfo : VpnSettingsDialog

    data object ContentBlockersInfo : VpnSettingsDialog

    data object CustomDnsInfo : VpnSettingsDialog

    data object MalwareInfo : VpnSettingsDialog

    data object ObfuscationInfo : VpnSettingsDialog

    data object QuantumResistanceInfo : VpnSettingsDialog

    data class WireguardPortInfo(val availablePortRanges: List<PortRange> = emptyList()) :
        VpnSettingsDialog

    data class CustomPort(val availablePortRanges: List<PortRange> = emptyList()) :
        VpnSettingsDialog
}
