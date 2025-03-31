package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.IpVersion
import net.mullvad.mullvadvpn.lib.model.Mtu
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.model.PortRange
import net.mullvad.mullvadvpn.lib.model.QuantumResistantState
import net.mullvad.mullvadvpn.viewmodel.CustomDnsItem

sealed interface VpnSettingItem {

    // Not available on TV devices
    data object AutoConnectAndLockdownModeHeader : VpnSettingItem

    data object AutoConnectAndLockdownModeInfo : VpnSettingItem

    // Only used on TV devices
    data class ConnectDeviceOnStartUpHeader(val enabled: Boolean) : VpnSettingItem

    data object ConnectDeviceOnStartUpInfo : VpnSettingItem

    data class LocalNetworkSharingHeader(val enabled: Boolean) : VpnSettingItem

    data class DnsContentBlockers(val featureEnabled: Boolean, val expanded: Boolean) :
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

    data class CustomDnsServerHeader(val enabled: Boolean, val isOptionEnabled: Boolean) :
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

    data class WireguardPortHeader(val enabled: Boolean, val availablePortRanges: List<PortRange>) :
        VpnSettingItem

    sealed interface WireguardPortItem : VpnSettingItem {
        val enabled: Boolean
        val selected: Boolean

        data class Constraint(
            override val enabled: Boolean,
            override val selected: Boolean,
            val constraint: net.mullvad.mullvadvpn.lib.model.Constraint<Port>,
        ) : WireguardPortItem

        data class WireguardPortCustom(
            override val enabled: Boolean,
            override val selected: Boolean,
            val customPort: Port?,
            val availablePortRanges: List<PortRange>,
        ) : WireguardPortItem
    }

    data object WireguardPortUnavailable : VpnSettingItem

    data object ObfuscationHeader : VpnSettingItem

    sealed interface ObfuscationItem : VpnSettingItem {
        val selected: Boolean

        data class Automatic(override val selected: Boolean) : ObfuscationItem

        data class Shadowsocks(override val selected: Boolean, val port: Constraint<Port>) :
            ObfuscationItem

        data class UdpOverTcp(override val selected: Boolean, val port: Constraint<Port>) :
            ObfuscationItem

        data class Off(override val selected: Boolean) : ObfuscationItem
    }

    data object QuantumResistanceHeader : VpnSettingItem

    data class QuantumItem(
        val quantumResistantState: QuantumResistantState,
        val selected: Boolean,
    ) : VpnSettingItem

    data object DeviceIpVersionHeader : VpnSettingItem

    data class DeviceIpVersionItem(val constraint: Constraint<IpVersion>, val selected: Boolean) :
        VpnSettingItem

    data object DeviceIpVersionInfo : VpnSettingItem

    data class EnableIpv6Header(val enabled: Boolean) : VpnSettingItem

    data class MtuHeader(val mtu: Mtu?) : VpnSettingItem

    data object MtuInfo : VpnSettingItem

    data object ServerIpOverridesHeader : VpnSettingItem

    data object Divider : VpnSettingItem

    data object Spacer : VpnSettingItem
}
