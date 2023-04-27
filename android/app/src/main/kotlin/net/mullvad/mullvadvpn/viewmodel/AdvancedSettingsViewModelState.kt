package net.mullvad.mullvadvpn.viewmodel

import net.mullvad.mullvadvpn.compose.state.AdvancedSettingsUiState
import net.mullvad.mullvadvpn.model.DefaultDnsOptions
import net.mullvad.mullvadvpn.model.SelectedObfuscation

data class AdvancedSettingsViewModelState(
    val mtuValue: String,
    val isCustomDnsEnabled: Boolean,
    val isAllowLanEnabled: Boolean,
    val customDnsList: List<CustomDnsItem>,
    val contentBlockersOptions: DefaultDnsOptions,
    val selectedObfuscation: SelectedObfuscation,
    val dialogState: AdvancedSettingsDialogState
) {
    fun toUiState(): AdvancedSettingsUiState {
        return when (dialogState) {
            is AdvancedSettingsDialogState.MtuDialog ->
                AdvancedSettingsUiState.MtuDialogUiState(
                    mtu = mtuValue,
                    isCustomDnsEnabled = isCustomDnsEnabled,
                    isAllowLanEnabled = isAllowLanEnabled,
                    customDnsItems = customDnsList,
                    contentBlockersOptions = contentBlockersOptions,
                    mtuEditValue = dialogState.mtuEditValue,
                    selectedObfuscation = selectedObfuscation
                )
            is AdvancedSettingsDialogState.DnsDialog ->
                AdvancedSettingsUiState.DnsDialogUiState(
                    mtu = mtuValue,
                    isCustomDnsEnabled = isCustomDnsEnabled,
                    isAllowLanEnabled = isAllowLanEnabled,
                    customDnsItems = customDnsList,
                    contentBlockersOptions = contentBlockersOptions,
                    stagedDns = dialogState.stagedDns,
                    selectedObfuscation = selectedObfuscation
                )
            is AdvancedSettingsDialogState.ContentBlockersInfoDialog ->
                AdvancedSettingsUiState.ContentBlockersInfoDialogUiState(
                    mtu = mtuValue,
                    isCustomDnsEnabled = isCustomDnsEnabled,
                    isAllowLanEnabled = isAllowLanEnabled,
                    customDnsItems = customDnsList,
                    contentBlockersOptions = contentBlockersOptions,
                    selectedObfuscation = selectedObfuscation
                )
            is AdvancedSettingsDialogState.CustomDnsInfoDialog ->
                AdvancedSettingsUiState.CustomDnsInfoDialogUiState(
                    mtu = mtuValue,
                    isCustomDnsEnabled = isCustomDnsEnabled,
                    isAllowLanEnabled = isAllowLanEnabled,
                    customDnsItems = customDnsList,
                    contentBlockersOptions = contentBlockersOptions
                )
            is AdvancedSettingsDialogState.MalwareInfoDialog ->
                AdvancedSettingsUiState.MalwareInfoDialogUiState(
                    mtu = mtuValue,
                    isCustomDnsEnabled = isCustomDnsEnabled,
                    isAllowLanEnabled = isAllowLanEnabled,
                    customDnsItems = customDnsList,
                    contentBlockersOptions = contentBlockersOptions,
                    selectedObfuscation = selectedObfuscation
                )
            is AdvancedSettingsDialogState.ObfuscationInfoDialog ->
                AdvancedSettingsUiState.ObfuscationInfoDialogUiState(
                    mtu = mtuValue,
                    isCustomDnsEnabled = isCustomDnsEnabled,
                    isAllowLanEnabled = isAllowLanEnabled,
                    customDnsItems = customDnsList,
                    contentBlockersOptions = contentBlockersOptions,
                    selectedObfuscation = selectedObfuscation
                )
            else ->
                AdvancedSettingsUiState.DefaultUiState(
                    mtu = mtuValue,
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
            AdvancedSettingsViewModelState(
                mtuValue = EMPTY_STRING,
                isCustomDnsEnabled = false,
                customDnsList = listOf(),
                contentBlockersOptions = DefaultDnsOptions(),
                isAllowLanEnabled = false,
                dialogState = AdvancedSettingsDialogState.NoDialog,
                selectedObfuscation = SelectedObfuscation.Auto
            )
    }
}

sealed class AdvancedSettingsDialogState {
    object NoDialog : AdvancedSettingsDialogState()

    data class MtuDialog(val mtuEditValue: String) : AdvancedSettingsDialogState()

    data class DnsDialog(val stagedDns: StagedDns) : AdvancedSettingsDialogState()

    object ContentBlockersInfoDialog : AdvancedSettingsDialogState()

    object CustomDnsInfoDialog : AdvancedSettingsDialogState()

    object MalwareInfoDialog : AdvancedSettingsDialogState()

    object ObfuscationInfoDialog : AdvancedSettingsDialogState()
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
