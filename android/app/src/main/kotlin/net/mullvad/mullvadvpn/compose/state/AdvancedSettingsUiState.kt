package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.model.DefaultDnsOptions
import net.mullvad.mullvadvpn.model.SelectedObfuscation
import net.mullvad.mullvadvpn.viewmodel.CustomDnsItem
import net.mullvad.mullvadvpn.viewmodel.StagedDns

sealed interface AdvancedSettingsUiState {
    val mtu: String
    val isCustomDnsEnabled: Boolean
    val customDnsItems: List<CustomDnsItem>
    val contentBlockersOptions: DefaultDnsOptions
    val isAllowLanEnabled: Boolean
    val selectedObfuscation: SelectedObfuscation

    data class DefaultUiState(
        override val mtu: String = "",
        override val isCustomDnsEnabled: Boolean = false,
        override val isAllowLanEnabled: Boolean = false,
        override val customDnsItems: List<CustomDnsItem> = listOf(),
        override val contentBlockersOptions: DefaultDnsOptions = DefaultDnsOptions(),
        override val selectedObfuscation: SelectedObfuscation = SelectedObfuscation.Off,
    ) : AdvancedSettingsUiState

    data class MtuDialogUiState(
        override val mtu: String = "",
        override val isCustomDnsEnabled: Boolean = false,
        override val isAllowLanEnabled: Boolean = false,
        override val customDnsItems: List<CustomDnsItem> = listOf(),
        override val contentBlockersOptions: DefaultDnsOptions = DefaultDnsOptions(),
        val mtuEditValue: String,
        override val selectedObfuscation: SelectedObfuscation = SelectedObfuscation.Off,
    ) : AdvancedSettingsUiState

    data class DnsDialogUiState(
        override val mtu: String = "",
        override val isCustomDnsEnabled: Boolean = false,
        override val isAllowLanEnabled: Boolean = false,
        override val customDnsItems: List<CustomDnsItem> = listOf(),
        override val contentBlockersOptions: DefaultDnsOptions = DefaultDnsOptions(),
        val stagedDns: StagedDns,
        override val selectedObfuscation: SelectedObfuscation = SelectedObfuscation.Off,
    ) : AdvancedSettingsUiState

    data class ContentBlockersInfoDialogUiState(
        override val mtu: String = "",
        override val isCustomDnsEnabled: Boolean = false,
        override val isAllowLanEnabled: Boolean = false,
        override val customDnsItems: List<CustomDnsItem> = listOf(),
        override val contentBlockersOptions: DefaultDnsOptions = DefaultDnsOptions(),
        override val selectedObfuscation: SelectedObfuscation = SelectedObfuscation.Off,
    ) : AdvancedSettingsUiState

    data class CustomDnsInfoDialogUiState(
        override val mtu: String = "",
        override val isCustomDnsEnabled: Boolean = false,
        override val isAllowLanEnabled: Boolean = false,
        override val customDnsItems: List<CustomDnsItem> = listOf(),
        override val contentBlockersOptions: DefaultDnsOptions = DefaultDnsOptions(),
        override val selectedObfuscation: SelectedObfuscation = SelectedObfuscation.Off,
    ) : AdvancedSettingsUiState

    data class MalwareInfoDialogUiState(
        override val mtu: String = "",
        override val isCustomDnsEnabled: Boolean = false,
        override val isAllowLanEnabled: Boolean = false,
        override val customDnsItems: List<CustomDnsItem> = listOf(),
        override val contentBlockersOptions: DefaultDnsOptions = DefaultDnsOptions(),
        override val selectedObfuscation: SelectedObfuscation = SelectedObfuscation.Off,
    ) : AdvancedSettingsUiState

    data class ObfuscationInfoDialogUiState(
        override val mtu: String = "",
        override val isCustomDnsEnabled: Boolean = false,
        override val isAllowLanEnabled: Boolean = false,
        override val customDnsItems: List<CustomDnsItem> = listOf(),
        override val contentBlockersOptions: DefaultDnsOptions = DefaultDnsOptions(),
        override val selectedObfuscation: SelectedObfuscation = SelectedObfuscation.Off,
    ) : AdvancedSettingsUiState
}
