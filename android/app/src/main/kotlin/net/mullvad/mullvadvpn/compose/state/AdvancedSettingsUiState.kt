package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.viewmodel.CustomDnsItem
import net.mullvad.mullvadvpn.viewmodel.StagedDns

sealed interface AdvancedSettingsUiState {
    val mtu: String
    val isCustomDnsEnabled: Boolean
    val customDnsItems: List<CustomDnsItem>
    val isAllowLanEnabled: Boolean

    data class DefaultUiState(
        override val mtu: String = "",
        override val isCustomDnsEnabled: Boolean = false,
        override val isAllowLanEnabled: Boolean = false,
        override val customDnsItems: List<CustomDnsItem> = listOf()
    ) : AdvancedSettingsUiState

    data class MtuDialogUiState(
        override val mtu: String,
        override val isCustomDnsEnabled: Boolean,
        override val isAllowLanEnabled: Boolean,
        override val customDnsItems: List<CustomDnsItem>,
        val mtuEditValue: String
    ) : AdvancedSettingsUiState

    data class DnsDialogUiState(
        override val mtu: String,
        override val isCustomDnsEnabled: Boolean,
        override val isAllowLanEnabled: Boolean,
        override val customDnsItems: List<CustomDnsItem>,
        val stagedDns: StagedDns,
    ) : AdvancedSettingsUiState
}
