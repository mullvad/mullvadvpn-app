package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.IpVersion
import net.mullvad.mullvadvpn.lib.model.QuantumResistantState

sealed interface VpnSettingItem {

    data object AntiCensorshipHeader : VpnSettingItem

    // Not available on TV devices
    data object AutoConnectAndLockdownMode : VpnSettingItem

    data object AutoConnectAndLockdownModeInfo : VpnSettingItem

    // Only used on TV devices
    data class ConnectDeviceOnStartUpSetting(val enabled: Boolean) : VpnSettingItem

    data class LocalNetworkSharingSetting(val enabled: Boolean) : VpnSettingItem

    data class DnsContentBlockersHeader(val featureEnabled: Boolean, val expanded: Boolean) :
        VpnSettingItem

    sealed interface DnsContentBlockerItem : VpnSettingItem {
        val enabled: Boolean
        val featureEnabled: Boolean

        data class Ads(override val enabled: Boolean, override val featureEnabled: Boolean) :
            DnsContentBlockerItem

        data class Trackers(override val enabled: Boolean, override val featureEnabled: Boolean) :
            DnsContentBlockerItem

        data class Malware(override val enabled: Boolean, override val featureEnabled: Boolean) :
            DnsContentBlockerItem

        data class Gambling(override val enabled: Boolean, override val featureEnabled: Boolean) :
            DnsContentBlockerItem

        data class AdultContent(
            override val enabled: Boolean,
            override val featureEnabled: Boolean,
        ) : DnsContentBlockerItem

        data class SocialMedia(
            override val enabled: Boolean,
            override val featureEnabled: Boolean,
        ) : DnsContentBlockerItem
    }

    data object DnsContentBlockersUnavailable : VpnSettingItem

    data class CustomDnsServerSetting(val enabled: Boolean, val isOptionEnabled: Boolean) :
        VpnSettingItem

    data class CustomDnsEntry(
        val index: Int,
        val customDnsItem: CustomDnsItem,
        val showUnreachableLocalDnsWarning: Boolean,
        val showUnreachableIpv6DnsWarning: Boolean,
    ) : VpnSettingItem

    data object CustomDnsAdd : VpnSettingItem

    data object CustomDnsUnavailable : VpnSettingItem

    data object CustomDnsInfo : VpnSettingItem

    data class EnableIpv6Setting(val enabled: Boolean) : VpnSettingItem

    data object QuantumResistanceHeader : VpnSettingItem

    data class QuantumItem(
        val quantumResistantState: QuantumResistantState,
        val selected: Boolean,
    ) : VpnSettingItem

    data object DeviceIpVersionHeader : VpnSettingItem

    data class DeviceIpVersionItem(val constraint: Constraint<IpVersion>, val selected: Boolean) :
        VpnSettingItem

    data class Mtu(val mtu: net.mullvad.mullvadvpn.lib.model.Mtu?) : VpnSettingItem

    data object MtuInfo : VpnSettingItem

    data object ServerIpOverrides : VpnSettingItem

    data object Divider : VpnSettingItem

    data object Spacer : VpnSettingItem
}
