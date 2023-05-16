package net.mullvad.mullvadvpn.viewmodel

import net.mullvad.mullvadvpn.compose.state.VpnSettingsUiState
import net.mullvad.mullvadvpn.model.DefaultDnsOptions
import net.mullvad.mullvadvpn.model.SelectedObfuscation

data class VpnSettingsViewModelState(
    val mtuValue: String,
    val isAutoConnectEnabled: Boolean,
    val isLocalNetworkSharingEnabled: Boolean,
    val isCustomDnsEnabled: Boolean,
    val isAllowLanEnabled: Boolean,
    val customDnsList: List<CustomDnsItem>,
    val contentBlockersOptions: DefaultDnsOptions,
    val selectedObfuscation: SelectedObfuscation,
    val dialogState: VpnSettingsDialogState
) {
    fun toUiState(): VpnSettingsUiState {
        return when (dialogState) {
            is VpnSettingsDialogState.MtuDialog ->
                VpnSettingsUiState.MtuDialogUiState(
                    mtu = mtuValue,
                    isAutoConnectEnabled = isAutoConnectEnabled,
                    isLocalNetworkSharingEnabled = isLocalNetworkSharingEnabled,
                    isCustomDnsEnabled = isCustomDnsEnabled,
                    isAllowLanEnabled = isAllowLanEnabled,
                    customDnsItems = customDnsList,
                    contentBlockersOptions = contentBlockersOptions,
                    mtuEditValue = dialogState.mtuEditValue,
                    selectedObfuscation = selectedObfuscation
                )
            is VpnSettingsDialogState.DnsDialog ->
                VpnSettingsUiState.DnsDialogUiState(
                    mtu = mtuValue,
                    isAutoConnectEnabled = isAutoConnectEnabled,
                    isLocalNetworkSharingEnabled = isLocalNetworkSharingEnabled,
                    isCustomDnsEnabled = isCustomDnsEnabled,
                    isAllowLanEnabled = isAllowLanEnabled,
                    customDnsItems = customDnsList,
                    contentBlockersOptions = contentBlockersOptions,
                    stagedDns = dialogState.stagedDns,
                    selectedObfuscation = selectedObfuscation
                )
            is VpnSettingsDialogState.LocalNetworkSharingInfoDialog ->
                VpnSettingsUiState.LocalNetworkSharingInfoDialogUiState(
                    mtu = mtuValue,
                    isAutoConnectEnabled = isAutoConnectEnabled,
                    isLocalNetworkSharingEnabled = isLocalNetworkSharingEnabled,
                    isCustomDnsEnabled = isCustomDnsEnabled,
                    isAllowLanEnabled = isAllowLanEnabled,
                    customDnsItems = customDnsList,
                    contentBlockersOptions = contentBlockersOptions
                )
            is VpnSettingsDialogState.ContentBlockersInfoDialog ->
                VpnSettingsUiState.ContentBlockersInfoDialogUiState(
                    mtu = mtuValue,
                    isAutoConnectEnabled = isAutoConnectEnabled,
                    isLocalNetworkSharingEnabled = isLocalNetworkSharingEnabled,
                    isCustomDnsEnabled = isCustomDnsEnabled,
                    isAllowLanEnabled = isAllowLanEnabled,
                    customDnsItems = customDnsList,
                    contentBlockersOptions = contentBlockersOptions,
                    selectedObfuscation = selectedObfuscation
                )
            is VpnSettingsDialogState.CustomDnsInfoDialog ->
                VpnSettingsUiState.CustomDnsInfoDialogUiState(
                    mtu = mtuValue,
                    isAutoConnectEnabled = isAutoConnectEnabled,
                    isLocalNetworkSharingEnabled = isLocalNetworkSharingEnabled,
                    isCustomDnsEnabled = isCustomDnsEnabled,
                    isAllowLanEnabled = isAllowLanEnabled,
                    customDnsItems = customDnsList,
                    contentBlockersOptions = contentBlockersOptions
                )
            is VpnSettingsDialogState.MalwareInfoDialog ->
                VpnSettingsUiState.MalwareInfoDialogUiState(
                    mtu = mtuValue,
                    isAutoConnectEnabled = isAutoConnectEnabled,
                    isLocalNetworkSharingEnabled = isLocalNetworkSharingEnabled,
                    isCustomDnsEnabled = isCustomDnsEnabled,
                    isAllowLanEnabled = isAllowLanEnabled,
                    customDnsItems = customDnsList,
                    contentBlockersOptions = contentBlockersOptions,
                    selectedObfuscation = selectedObfuscation
                )
            is VpnSettingsDialogState.ObfuscationInfoDialog ->
                VpnSettingsUiState.ObfuscationInfoDialogUiState(
                    mtu = mtuValue,
                    isCustomDnsEnabled = isCustomDnsEnabled,
                    isAllowLanEnabled = isAllowLanEnabled,
                    customDnsItems = customDnsList,
                    contentBlockersOptions = contentBlockersOptions,
                    selectedObfuscation = selectedObfuscation
                )
            else ->
                VpnSettingsUiState.DefaultUiState(
                    mtu = mtuValue,
                    isAutoConnectEnabled = isAutoConnectEnabled,
                    isLocalNetworkSharingEnabled = isLocalNetworkSharingEnabled,
                    isCustomDnsEnabled = isCustomDnsEnabled,
                    isAllowLanEnabled = isAllowLanEnabled,
                    customDnsItems = customDnsList,
                    contentBlockersOptions = contentBlockersOptions,
                    selectedObfuscation = selectedObfuscation
                )
        }
    }

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
                isAllowLanEnabled = false,
                dialogState = VpnSettingsDialogState.NoDialog,
                selectedObfuscation = SelectedObfuscation.Auto
            )
    }
}

sealed class VpnSettingsDialogState {
    object NoDialog : VpnSettingsDialogState()

    data class MtuDialog(val mtuEditValue: String) : VpnSettingsDialogState()

    data class DnsDialog(val stagedDns: StagedDns) : VpnSettingsDialogState()

    object LocalNetworkSharingInfoDialog : VpnSettingsDialogState()

    object ContentBlockersInfoDialog : VpnSettingsDialogState()

    object CustomDnsInfoDialog : VpnSettingsDialogState()

    object MalwareInfoDialog : VpnSettingsDialogState()

    object ObfuscationInfoDialog : VpnSettingsDialogState()
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
        object Success : ValidationResult()
        object InvalidAddress : ValidationResult()
        object DuplicateAddress : ValidationResult()
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
