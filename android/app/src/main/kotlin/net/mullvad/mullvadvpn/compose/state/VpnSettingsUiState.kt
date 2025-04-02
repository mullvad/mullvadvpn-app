package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.DefaultDnsOptions
import net.mullvad.mullvadvpn.lib.model.IpVersion
import net.mullvad.mullvadvpn.lib.model.Mtu
import net.mullvad.mullvadvpn.lib.model.ObfuscationMode
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

    data class DnsContentBlockers(val enabled: Boolean, val expanded: Boolean) : VpnSettingItem

    sealed interface DnsContentBlockerItem : VpnSettingItem {
        val enabled: Boolean

        data class Ads(override val enabled: Boolean) : DnsContentBlockerItem

        data class Trackers(override val enabled: Boolean) : DnsContentBlockerItem

        data class Malware(override val enabled: Boolean) : DnsContentBlockerItem

        data class Gambling(override val enabled: Boolean) : DnsContentBlockerItem

        data class AdultContent(override val enabled: Boolean) : DnsContentBlockerItem

        data class SocialMedia(override val enabled: Boolean) : DnsContentBlockerItem
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

    data class WireguardPortHeader(val enabled: Boolean, val availablePortRanges: List<PortRange>) : VpnSettingItem

    sealed interface WireguardPortItem : VpnSettingItem {
        val enabled: Boolean
        val selected: Boolean

        data class Automatic(
            override val enabled: Boolean,
            override val selected: Boolean,
        ) : WireguardPortItem

        data class FixedPort(
            override val enabled: Boolean,
            override val selected: Boolean,
            val port: Port,
        ) : WireguardPortItem

        data class WireguardPortCustom(
            override val enabled: Boolean,
            override val selected: Boolean,
            val customPort: Port?,
        ) : WireguardPortItem
    }

    data object WireguardPortUnavailable : VpnSettingItem

    data object ObfuscationHeader : VpnSettingItem

    sealed interface ObfuscationItem : VpnSettingItem {
        val selected: Boolean

        data class Automatic(
            override val selected: Boolean,
        ) : ObfuscationItem

        data class Shadowsocks(
            override val selected: Boolean,
            val port: Constraint<Port>,
        ) : ObfuscationItem

        data class UdpOverTcp(
            override val selected: Boolean,
            val port: Constraint<Port>,
        ) : ObfuscationItem

        data class Off(override val selected: Boolean) :
            ObfuscationItem
    }

    data object QuantumResistanceHeader : VpnSettingItem

    sealed interface QuantumItem : VpnSettingItem {
        val selected: Boolean

        data class Automatic(override val selected: Boolean) : QuantumItem

        data class On(override val selected: Boolean) : QuantumItem

        data class Off(override val selected: Boolean) : QuantumItem
    }

    data object DeviceIpVersionHeader : VpnSettingItem

    sealed interface DeviceIpVersionItem : VpnSettingItem {
        val selected: Boolean

        data class Automatic(override val selected: Boolean) : DeviceIpVersionItem

        data class Ip(override val selected: Boolean, val ipVersion: IpVersion) :
            DeviceIpVersionItem
    }

    data class EnableIpv6Header(val enabled: Boolean) : VpnSettingItem

    data class MtuHeader(val mtu: Mtu?) : VpnSettingItem

    data object MtuInfo : VpnSettingItem

    data object ServerIpOverridesHeader : VpnSettingItem

    data object Divider : VpnSettingItem

    data object Spacer : VpnSettingItem
}
