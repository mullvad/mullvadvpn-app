package net.mullvad.mullvadvpn.viewmodel

import net.mullvad.mullvadvpn.compose.state.VpnSettingItem
import net.mullvad.mullvadvpn.constant.WIREGUARD_PRESET_PORTS
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.DefaultDnsOptions
import net.mullvad.mullvadvpn.lib.model.IpVersion
import net.mullvad.mullvadvpn.lib.model.Mtu
import net.mullvad.mullvadvpn.lib.model.ObfuscationMode
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.model.PortRange
import net.mullvad.mullvadvpn.lib.model.QuantumResistantState

sealed interface VpnSettingsUiState {
    val isModal: Boolean

    data class Loading(override val isModal: Boolean = false) : VpnSettingsUiState

    data class Content(val settings: List<VpnSettingItem>, override val isModal: Boolean = false) :
        VpnSettingsUiState {
        companion object {
            @Suppress("LongParameterList", "CyclomaticComplexMethod", "LongMethod")
            fun from(
                mtu: Mtu?,
                isLocalNetworkSharingEnabled: Boolean,
                isCustomDnsEnabled: Boolean,
                customDnsItems: List<CustomDnsItem>,
                contentBlockersOptions: DefaultDnsOptions,
                obfuscationMode: ObfuscationMode,
                selectedUdp2TcpObfuscationPort: Constraint<Port>,
                selectedShadowsocksObfuscationPort: Constraint<Port>,
                quantumResistant: QuantumResistantState,
                selectedWireguardPort: Constraint<Port>,
                customWireguardPort: Port?,
                availablePortRanges: List<PortRange>,
                systemVpnSettingsAvailable: Boolean,
                autoStartAndConnectOnBoot: Boolean,
                deviceIpVersion: Constraint<IpVersion>,
                isIpv6Enabled: Boolean,
                isContentBlockersExpanded: Boolean,
                isModal: Boolean,
            ) =
                Content(
                    buildList {
                        if (systemVpnSettingsAvailable) {
                            add(VpnSettingItem.AutoConnectAndLockdownModeHeader)
                            add(VpnSettingItem.AutoConnectAndLockdownModeInfo)
                        } else {
                            add(
                                VpnSettingItem.ConnectDeviceOnStartUpHeader(
                                    autoStartAndConnectOnBoot
                                )
                            )
                            add(VpnSettingItem.ConnectDeviceOnStartUpInfo)
                        }
                        add(VpnSettingItem.Spacer)

                        // Local network sharing
                        add(VpnSettingItem.LocalNetworkSharingHeader(isLocalNetworkSharingEnabled))
                        add(VpnSettingItem.Spacer)

                        // Dns Content Blockers
                        add(
                            VpnSettingItem.DnsContentBlockers(
                                !isCustomDnsEnabled,
                                isContentBlockersExpanded,
                            )
                        )
                        add(VpnSettingItem.Divider)

                        if (isContentBlockersExpanded) {
                            with(contentBlockersOptions) {
                                add(
                                    VpnSettingItem.DnsContentBlockerItem.Ads(
                                        blockAds,
                                        !isCustomDnsEnabled,
                                    )
                                )
                                add(VpnSettingItem.Divider)
                                add(
                                    VpnSettingItem.DnsContentBlockerItem.Trackers(
                                        blockTrackers,
                                        !isCustomDnsEnabled,
                                    )
                                )
                                add(VpnSettingItem.Divider)
                                add(
                                    VpnSettingItem.DnsContentBlockerItem.Malware(
                                        blockMalware,
                                        !isCustomDnsEnabled,
                                    )
                                )
                                add(VpnSettingItem.Divider)
                                add(
                                    VpnSettingItem.DnsContentBlockerItem.Gambling(
                                        blockGambling,
                                        !isCustomDnsEnabled,
                                    )
                                )
                                add(VpnSettingItem.Divider)
                                add(
                                    VpnSettingItem.DnsContentBlockerItem.AdultContent(
                                        blockAdultContent,
                                        !isCustomDnsEnabled,
                                    )
                                )
                                add(VpnSettingItem.Divider)
                                add(
                                    VpnSettingItem.DnsContentBlockerItem.SocialMedia(
                                        blockSocialMedia,
                                        !isCustomDnsEnabled,
                                    )
                                )
                            }
                            if (isCustomDnsEnabled) {
                                add(VpnSettingItem.DnsContentBlockersUnavailable)
                                add(VpnSettingItem.Spacer)
                            }
                        }

                        // Custom DNS
                        add(
                            VpnSettingItem.CustomDnsServerHeader(
                                isCustomDnsEnabled,
                                !contentBlockersOptions.isAnyBlockerEnabled(),
                            )
                        )
                        if (isCustomDnsEnabled) {
                            customDnsItems.forEachIndexed { index, item ->
                                add(
                                    VpnSettingItem.CustomDnsEntry(
                                        index,
                                        item,
                                        showUnreachableLocalDnsWarning =
                                            item.isLocal && !isLocalNetworkSharingEnabled,
                                        showUnreachableIpv6DnsWarning =
                                            item.isIpv6 && !isIpv6Enabled,
                                    )
                                )
                                add(VpnSettingItem.Divider)
                            }
                            if (customDnsItems.isNotEmpty()) {
                                add(VpnSettingItem.CustomDnsAdd)
                            }
                        }

                        if (contentBlockersOptions.isAnyBlockerEnabled()) {
                            add(VpnSettingItem.CustomDnsUnavailable)
                        } else {
                            add(VpnSettingItem.CustomDnsInfo)
                        }
                        add(VpnSettingItem.Spacer)

                        // IPv6
                        add(VpnSettingItem.EnableIpv6Header(isIpv6Enabled))

                        add(VpnSettingItem.Spacer)

                        // Wireguard Port
                        val isWireguardPortEnabled =
                            obfuscationMode == ObfuscationMode.Auto ||
                                obfuscationMode == ObfuscationMode.Off
                        add(
                            VpnSettingItem.WireguardPortHeader(
                                isWireguardPortEnabled,
                                availablePortRanges,
                            )
                        )
                        (listOf(Constraint.Any) +
                                WIREGUARD_PRESET_PORTS.map { Constraint.Only(it) })
                            .forEach {
                                add(VpnSettingItem.Divider)
                                add(
                                    VpnSettingItem.WireguardPortItem.Constraint(
                                        isWireguardPortEnabled,
                                        it == selectedWireguardPort,
                                        it,
                                    )
                                )
                            }
                        add(VpnSettingItem.Divider)
                        add(
                            VpnSettingItem.WireguardPortItem.WireguardPortCustom(
                                isWireguardPortEnabled,
                                selectedWireguardPort is Constraint.Only &&
                                    selectedWireguardPort.value == customWireguardPort,
                                customWireguardPort,
                                availablePortRanges,
                            )
                        )

                        if (!isWireguardPortEnabled) {
                            add(VpnSettingItem.WireguardPortUnavailable)
                        }
                        add(VpnSettingItem.Spacer)

                        // Wireguard Obfuscation
                        add(VpnSettingItem.ObfuscationHeader)
                        add(VpnSettingItem.Divider)
                        add(
                            VpnSettingItem.ObfuscationItem.Automatic(
                                obfuscationMode == ObfuscationMode.Auto
                            )
                        )
                        add(VpnSettingItem.Divider)
                        add(
                            VpnSettingItem.ObfuscationItem.Shadowsocks(
                                obfuscationMode == ObfuscationMode.Shadowsocks,
                                selectedShadowsocksObfuscationPort,
                            )
                        )
                        add(VpnSettingItem.Divider)
                        add(
                            VpnSettingItem.ObfuscationItem.UdpOverTcp(
                                obfuscationMode == ObfuscationMode.Udp2Tcp,
                                selectedUdp2TcpObfuscationPort,
                            )
                        )
                        add(VpnSettingItem.Divider)
                        add(
                            VpnSettingItem.ObfuscationItem.Off(
                                obfuscationMode == ObfuscationMode.Off
                            )
                        )

                        add(VpnSettingItem.Spacer)

                        // Quantum Resistance
                        add(VpnSettingItem.QuantumResistanceHeader)
                        QuantumResistantState.entries.forEach {
                            add(VpnSettingItem.Divider)
                            add(VpnSettingItem.QuantumItem(it, quantumResistant == it))
                        }

                        add(VpnSettingItem.Spacer)

                        // Device Ip Version
                        add(VpnSettingItem.DeviceIpVersionHeader)

                        IpVersion.constraints.forEach {
                            add(VpnSettingItem.Divider)
                            add(VpnSettingItem.DeviceIpVersionItem(it, deviceIpVersion == it))
                        }

                        add(VpnSettingItem.DeviceIpVersionInfo)
                        add(VpnSettingItem.Spacer)

                        // MTU
                        add(VpnSettingItem.MtuHeader(mtu))
                        add(VpnSettingItem.MtuInfo)
                        add(VpnSettingItem.Spacer)

                        add(VpnSettingItem.ServerIpOverridesHeader)
                        add(VpnSettingItem.Spacer)
                    },
                    isModal = isModal,
                )
        }
    }
}

data class CustomDnsItem(val address: String, val isLocal: Boolean, val isIpv6: Boolean) {
    companion object {
        private const val EMPTY_STRING = ""

        fun default(): CustomDnsItem {
            return CustomDnsItem(address = EMPTY_STRING, isLocal = false, isIpv6 = false)
        }
    }
}
