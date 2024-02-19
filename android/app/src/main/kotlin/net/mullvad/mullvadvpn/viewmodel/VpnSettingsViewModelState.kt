package net.mullvad.mullvadvpn.viewmodel

import net.mullvad.mullvadvpn.compose.state.VpnSettingsUiState
import net.mullvad.mullvadvpn.model.Constraint
import net.mullvad.mullvadvpn.model.DefaultDnsOptions
import net.mullvad.mullvadvpn.model.Port
import net.mullvad.mullvadvpn.model.PortRange
import net.mullvad.mullvadvpn.model.QuantumResistantState
import net.mullvad.mullvadvpn.model.SelectedObfuscation

data class VpnSettingsViewModelState(
    val mtuValue: String,
    val isAutoConnectEnabled: Boolean,
    val isLocalNetworkSharingEnabled: Boolean,
    val isCustomDnsEnabled: Boolean,
    val customDnsList: List<CustomDnsItem>,
    val contentBlockersOptions: DefaultDnsOptions,
    val selectedObfuscation: SelectedObfuscation,
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
            isCustomDnsEnabled,
            customDnsList,
            contentBlockersOptions,
            selectedObfuscation,
            quantumResistant,
            selectedWireguardPort,
            customWireguardPort,
            availablePortRanges,
            systemVpnSettingsAvailable
        )

    companion object {
        private const val EMPTY_STRING = ""

        fun default() =
            VpnSettingsViewModelState(
                mtuValue = EMPTY_STRING,
                isAutoConnectEnabled = false,
                isLocalNetworkSharingEnabled = false,
                isCustomDnsEnabled = false,
                customDnsList = listOf(),
                contentBlockersOptions = DefaultDnsOptions(),
                selectedObfuscation = SelectedObfuscation.Auto,
                quantumResistant = QuantumResistantState.Off,
                selectedWireguardPort = Constraint.Any(),
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
