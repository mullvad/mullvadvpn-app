package net.mullvad.mullvadvpn.test.common.page

import net.mullvad.mullvadvpn.test.common.extension.pressBackTwice

// This file defines extension methods on Page objects that involve multiple actions
// that navigate multiple pages.

fun ConnectPage.disablePostQuantumStory() {
    clickSettings()
    on<SettingsPage> { clickVpnSettings() }
    on<VpnSettingsPage> {
        scrollUntilPostQuantumOffCell()
        clickPostQuantumOffCell()
    }
    uiDevice.pressBackTwice()
}

fun ConnectPage.enablePostQuantumStory() {
    clickSettings()
    on<SettingsPage> { clickVpnSettings() }
    on<VpnSettingsPage> {
        scrollUntilPostQuantumOnCell()
        clickPostQuantumOnCell()
    }
    uiDevice.pressBackTwice()
}

enum class ObfuscationOption {
    WireguardPort,
    Udp2Tcp,
    Shadowsocks,
    Quic,
    Lwo,
    Off,
}

fun ConnectPage.setObfuscationStory(obfuscation: ObfuscationOption) {
    clickSettings()
    on<SettingsPage> { clickVpnSettings() }
    on<VpnSettingsPage> {
        scrollUntilAntiCensorshipCell()
        clickAntiCensorshipCell()
    }
    on<AntiCensorshipSettingsPage> {
        when (obfuscation) {
            ObfuscationOption.WireguardPort -> clickWireguardPortCell()
            ObfuscationOption.Udp2Tcp -> clickUdp2TcpCell()
            ObfuscationOption.Shadowsocks -> clickShadowsocksCell()
            ObfuscationOption.Quic -> clickQuicCell()
            ObfuscationOption.Lwo -> clickLwoCell()
            ObfuscationOption.Off -> clickObfuscationOffCell()
        }
    }
    repeat(3) { uiDevice.pressBack() }
}

fun ConnectPage.enableDAITAStory() {
    clickSettings()
    on<SettingsPage> { clickDaita() }
    on<DaitaSettingsPage> { clickEnableSwitch() }
    uiDevice.pressBackTwice()
}

fun ConnectPage.enableLocalNetworkSharingStory() {
    clickSettings()
    on<SettingsPage> { clickVpnSettings() }
    on<VpnSettingsPage> { clickLocalNetworkSharingSwitch() }
    uiDevice.pressBackTwice()
}

fun ConnectPage.toggleInTunnelIpv6Story() {
    clickSettings()
    on<SettingsPage> { clickVpnSettings() }
    on<VpnSettingsPage> { clickInTunnelIpv6Switch() }
    uiDevice.pressBackTwice()
}

fun ConnectPage.enableWireGuardCustomPortStory(port: Int) {
    if (port != 51820 && port != 53) {
        error("Port needs to be one of the predefined ports")
    }
    clickSettings()
    on<SettingsPage> { clickVpnSettings() }
    on<VpnSettingsPage> {
        scrollUntilAntiCensorshipCell()
        clickAntiCensorshipCell()
    }
    on<AntiCensorshipSettingsPage> { clickWireguardSelectPortButton() }
    on<SelectPortPage> { clickPresetPort(port) }
    uiDevice.pressBack()
    on<AntiCensorshipSettingsPage> { clickWireguardPortCell() }
    repeat(3) { uiDevice.pressBack() }
}
