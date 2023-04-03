package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.model.DefaultDnsOptions
import net.mullvad.mullvadvpn.viewmodel.CustomDnsItem
import net.mullvad.mullvadvpn.viewmodel.StagedDns

sealed interface AdvancedSettingsUiState {
    val mtu: String
    val isCustomDnsEnabled: Boolean
    val isCustomDnsClickable: Boolean
    val customDnsItems: List<CustomDnsItem>
    val contentBlockersOptions: DefaultDnsOptions
    val isAllowLanEnabled: Boolean

    data class DefaultUiState(
        override val mtu: String = "",
        override val isCustomDnsEnabled: Boolean = false,
        override val isCustomDnsClickable: Boolean = false,
        override val isAllowLanEnabled: Boolean = false,
        override val customDnsItems: List<CustomDnsItem> = listOf(),
        override val contentBlockersOptions: DefaultDnsOptions = DefaultDnsOptions(),
    ) : AdvancedSettingsUiState

    data class MtuDialogUiState(
        override val mtu: String = "",
        override val isCustomDnsEnabled: Boolean = false,
        override val isCustomDnsClickable: Boolean = false,
        override val isAllowLanEnabled: Boolean = false,
        override val customDnsItems: List<CustomDnsItem> = listOf(),
        override val contentBlockersOptions: DefaultDnsOptions = DefaultDnsOptions(),
        val mtuEditValue: String
    ) : AdvancedSettingsUiState

    data class DnsDialogUiState(
        override val mtu: String = "",
        override val isCustomDnsEnabled: Boolean = false,
        override val isCustomDnsClickable: Boolean = false,
        override val isAllowLanEnabled: Boolean = false,
        override val customDnsItems: List<CustomDnsItem> = listOf(),
        override val contentBlockersOptions: DefaultDnsOptions = DefaultDnsOptions(),
        val stagedDns: StagedDns,
    ) : AdvancedSettingsUiState

    data class ContentBlockersInfoDialogUiState(
        override val mtu: String = "",
        override val isCustomDnsEnabled: Boolean = false,
        override val isCustomDnsClickable: Boolean = false,
        override val isAllowLanEnabled: Boolean = false,
        override val customDnsItems: List<CustomDnsItem> = listOf(),
        override val contentBlockersOptions: DefaultDnsOptions = DefaultDnsOptions(),
    ) : AdvancedSettingsUiState

    data class MalwareInfoDialogUiState(
        override val mtu: String = "",
        override val isCustomDnsEnabled: Boolean = false,
        override val isCustomDnsClickable: Boolean = false,
        override val isAllowLanEnabled: Boolean = false,
        override val customDnsItems: List<CustomDnsItem> = listOf(),
        override val contentBlockersOptions: DefaultDnsOptions = DefaultDnsOptions(),
    ) : AdvancedSettingsUiState
}
