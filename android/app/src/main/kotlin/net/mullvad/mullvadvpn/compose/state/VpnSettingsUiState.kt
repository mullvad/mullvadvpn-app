package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.DefaultDnsOptions
import net.mullvad.mullvadvpn.lib.model.IpVersion
import net.mullvad.mullvadvpn.lib.model.Mtu
import net.mullvad.mullvadvpn.lib.model.ObfuscationMode
import net.mullvad.mullvadvpn.lib.model.QuantumResistantState

data class VpnSettingsUiState(
    val settings: List<VpnSettingItem>,
    val isModal: Boolean,
    val isScrollToFeatureEnabled: Boolean,
    val obfuscationMode: ObfuscationMode,
) {

    companion object {
        @Suppress("LongParameterList", "CyclomaticComplexMethod", "LongMethod")
        fun from(
            mtu: Mtu?,
            isLocalNetworkSharingEnabled: Boolean,
            isCustomDnsEnabled: Boolean,
            customDnsItems: List<CustomDnsItem>,
            contentBlockersOptions: DefaultDnsOptions,
            obfuscationMode: ObfuscationMode,
            quantumResistant: QuantumResistantState,
            systemVpnSettingsAvailable: Boolean,
            autoStartAndConnectOnBoot: Boolean,
            deviceIpVersion: Constraint<IpVersion>,
            isIpv6Enabled: Boolean,
            isContentBlockersExpanded: Boolean,
            isModal: Boolean,
            isScrollToFeatureEnabled: Boolean,
        ) =
            VpnSettingsUiState(
                buildList {
                    if (systemVpnSettingsAvailable) {
                        add(VpnSettingItem.AutoConnectAndLockdownMode)
                        add(VpnSettingItem.AutoConnectAndLockdownModeInfo)
                    } else {
                        add(VpnSettingItem.ConnectDeviceOnStartUpSetting(autoStartAndConnectOnBoot))
                        add(VpnSettingItem.Spacer)
                    }

                    // Local network sharing
                    add(VpnSettingItem.LocalNetworkSharingSetting(isLocalNetworkSharingEnabled))
                    add(VpnSettingItem.Spacer)

                    // Dns Content Blockers
                    add(
                        VpnSettingItem.DnsContentBlockersHeader(
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
                        }
                    }
                    add(VpnSettingItem.Spacer)

                    // Custom DNS
                    add(
                        VpnSettingItem.CustomDnsServerSetting(
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
                                    showUnreachableIpv6DnsWarning = item.isIpv6 && !isIpv6Enabled,
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
                    } else if (customDnsItems.isEmpty()) {
                        add(VpnSettingItem.CustomDnsInfo)
                    } else {
                        add(VpnSettingItem.Spacer)
                    }

                    // IPv6
                    add(VpnSettingItem.EnableIpv6Setting(isIpv6Enabled))

                    add(VpnSettingItem.Spacer)

                    // Anti-censorship
                    add(VpnSettingItem.AntiCensorshipHeader)

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

                    add(VpnSettingItem.Spacer)

                    // MTU
                    add(VpnSettingItem.Mtu(mtu))
                    add(VpnSettingItem.MtuInfo)

                    // Server IP override
                    add(VpnSettingItem.ServerIpOverrides)
                    add(VpnSettingItem.Spacer)
                },
                isModal = isModal,
                isScrollToFeatureEnabled = isScrollToFeatureEnabled,
                obfuscationMode = obfuscationMode,
            )
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
