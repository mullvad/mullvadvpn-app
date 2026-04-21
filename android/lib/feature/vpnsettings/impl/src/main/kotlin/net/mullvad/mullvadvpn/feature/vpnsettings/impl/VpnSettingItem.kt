package net.mullvad.mullvadvpn.feature.vpnsettings.impl

import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.IpVersion

sealed interface VpnSettingItem {

    data object AntiCensorshipHeader : VpnSettingItem

    data object DnsHeader : VpnSettingItem

    // Not available on TV devices
    data object AutoConnectAndLockdownMode : VpnSettingItem

    data object AutoConnectAndLockdownModeInfo : VpnSettingItem

    // Only used on TV devices
    data class ConnectDeviceOnStartUpSetting(val enabled: Boolean) : VpnSettingItem

    data class LocalNetworkSharingSetting(val enabled: Boolean) : VpnSettingItem

    data class EnableIpv6Setting(val enabled: Boolean) : VpnSettingItem

    data class QuantumResistantSetting(val enabled: Boolean) : VpnSettingItem

    data object DeviceIpVersionHeader : VpnSettingItem

    data class DeviceIpVersionItem(val constraint: Constraint<IpVersion>, val selected: Boolean) :
        VpnSettingItem

    data class Mtu(val mtu: net.mullvad.mullvadvpn.lib.model.Mtu?) : VpnSettingItem

    data object ServerIpOverrides : VpnSettingItem

    data object Divider : VpnSettingItem

    data object Spacer : VpnSettingItem

    data object SmallSpacer : VpnSettingItem
}
