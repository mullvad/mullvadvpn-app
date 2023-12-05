package net.mullvad.mullvadvpn.viewmodel

import net.mullvad.mullvadvpn.compose.state.VpnSettingsDialog
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
    val isConnectOnBootEnabled: Boolean?,
    val isLocalNetworkSharingEnabled: Boolean,
    val isCustomDnsEnabled: Boolean,
    val isAllowLanEnabled: Boolean,
    val customDnsList: List<CustomDnsItem>,
    val contentBlockersOptions: DefaultDnsOptions,
    val selectedObfuscation: SelectedObfuscation,
    val quantumResistant: QuantumResistantState,
    val selectedWireguardPort: Constraint<Port>,
    val availablePortRanges: List<PortRange>,
    val dialogState: VpnSettingsDialogState?,
) {
    fun toUiState(): VpnSettingsUiState =
        VpnSettingsUiState(
            mtuValue,
            isAutoConnectEnabled,
            isConnectOnBootEnabled,
            isLocalNetworkSharingEnabled,
            isCustomDnsEnabled,
            customDnsList,
            contentBlockersOptions,
            isAllowLanEnabled,
            selectedObfuscation,
            quantumResistant,
            selectedWireguardPort,
            availablePortRanges,
            dialogState.toUi(this@VpnSettingsViewModelState)
        )

    companion object {
        private const val EMPTY_STRING = ""

        fun default() =
            VpnSettingsViewModelState(
                mtuValue = EMPTY_STRING,
                isAutoConnectEnabled = false,
                isConnectOnBootEnabled = null,
                isLocalNetworkSharingEnabled = false,
                isCustomDnsEnabled = false,
                customDnsList = listOf(),
                contentBlockersOptions = DefaultDnsOptions(),
                isAllowLanEnabled = false,
                dialogState = null,
                selectedObfuscation = SelectedObfuscation.Auto,
                quantumResistant = QuantumResistantState.Off,
                selectedWireguardPort = Constraint.Any(),
                availablePortRanges = emptyList()
            )
    }
}

private fun VpnSettingsDialogState?.toUi(
    vpnSettingsViewModelState: VpnSettingsViewModelState
): VpnSettingsDialog? =
    when (this) {
        VpnSettingsDialogState.ContentBlockersInfoDialog -> VpnSettingsDialog.ContentBlockersInfo
        VpnSettingsDialogState.CustomDnsInfoDialog -> VpnSettingsDialog.CustomDnsInfo
        VpnSettingsDialogState.CustomPortDialog ->
            VpnSettingsDialog.CustomPort(vpnSettingsViewModelState.availablePortRanges)
        is VpnSettingsDialogState.DnsDialog -> VpnSettingsDialog.Dns(stagedDns)
        VpnSettingsDialogState.LocalNetworkSharingInfoDialog ->
            VpnSettingsDialog.LocalNetworkSharingInfo
        VpnSettingsDialogState.MalwareInfoDialog -> VpnSettingsDialog.MalwareInfo
        is VpnSettingsDialogState.MtuDialog -> VpnSettingsDialog.Mtu(mtuEditValue)
        VpnSettingsDialogState.ObfuscationInfoDialog -> VpnSettingsDialog.ObfuscationInfo
        VpnSettingsDialogState.QuantumResistanceInfoDialog ->
            VpnSettingsDialog.QuantumResistanceInfo
        VpnSettingsDialogState.WireguardPortInfoDialog ->
            VpnSettingsDialog.WireguardPortInfo(vpnSettingsViewModelState.availablePortRanges)
        null -> null
    }

sealed class VpnSettingsDialogState {

    data class MtuDialog(val mtuEditValue: String) : VpnSettingsDialogState()

    data class DnsDialog(val stagedDns: StagedDns) : VpnSettingsDialogState()

    data object LocalNetworkSharingInfoDialog : VpnSettingsDialogState()

    data object ContentBlockersInfoDialog : VpnSettingsDialogState()

    data object CustomDnsInfoDialog : VpnSettingsDialogState()

    data object MalwareInfoDialog : VpnSettingsDialogState()

    data object ObfuscationInfoDialog : VpnSettingsDialogState()

    data object QuantumResistanceInfoDialog : VpnSettingsDialogState()

    data object WireguardPortInfoDialog : VpnSettingsDialogState()

    data object CustomPortDialog : VpnSettingsDialogState()
}

sealed interface StagedDns {
    val item: CustomDnsItem
    val validationResult: ValidationResult

    data class NewDns(
        override val item: CustomDnsItem,
        override val validationResult: ValidationResult = ValidationResult.Success,
    ) : StagedDns

    data class EditDns(
        override val item: CustomDnsItem,
        override val validationResult: ValidationResult = ValidationResult.Success,
        val index: Int
    ) : StagedDns

    sealed class ValidationResult {
        data object Success : ValidationResult()

        data object InvalidAddress : ValidationResult()

        data object DuplicateAddress : ValidationResult()
    }

    fun isValid() = (validationResult is ValidationResult.Success)
}

data class CustomDnsItem(val address: String, val isLocal: Boolean) {
    companion object {
        private const val EMPTY_STRING = ""

        fun default(): CustomDnsItem {
            return CustomDnsItem(address = EMPTY_STRING, isLocal = false)
        }
    }
}
