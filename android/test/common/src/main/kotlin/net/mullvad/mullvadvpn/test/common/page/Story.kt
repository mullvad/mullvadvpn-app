package net.mullvad.mullvadvpn.test.common.page

import net.mullvad.mullvadvpn.test.common.extension.pressBackThrice
import net.mullvad.mullvadvpn.test.common.extension.pressBackTwice

// This file defines extension methods on Page objects that involve multiple actions
// that navigate multiple pages.

fun ConnectPage.disableObfuscationStory() {
    clickSettings()
    on<SettingsPage> { clickVpnSettings() }
    on<VpnSettingsPage> {
        scrollUntilWireGuardObfuscationOffCell()
        clickWireGuardObfuscationOffCell()
    }
    uiDevice.pressBackTwice()
}

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

fun ConnectPage.enableShadowsocksStory() {
    clickSettings()
    on<SettingsPage> { clickVpnSettings() }
    on<VpnSettingsPage> {
        scrollUntilWireGuardObfuscationShadowsocksCell()
        clickWireGuardObfuscationShadowsocksCell()
    }
    uiDevice.pressBackTwice()
}

fun ConnectPage.enableDAITAStory() {
    clickSettings()
    on<SettingsPage> { clickDaita() }
    on<DaitaSettingsPage> { clickEnableSwitch() }
    uiDevice.pressBackTwice()
}

fun ConnectPage.enableMultihopStory() {
    clickSettings()
    on<SettingsPage> { clickMultihop() }
    on<MultihopSettingsPage> { clickEnableSwitch() }
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

fun ConnectPage.enableServerIpOverrideStory(relay: String, overrideIp: String) {
    clickSettings()
    on<SettingsPage> { clickVpnSettings() }
    on<VpnSettingsPage> {
        // Disable obfuscation
        scrollUntilWireGuardObfuscationOffCell()
        clickWireGuardObfuscationOffCell()
        // Disable IPv6
        scrollUntilDeviceIpVersionCell()
        clickDeviceIpIpv4Cell()
        // Open ServerIPOverrideScreen
        scrollUntilServerIpOverride()
        clickServerIpOverrideButton()
    }
    on<ServerIpOverridesPage> { clickImportButton() }
    on<ImportOverridesBottomSheet> { clickText() }
    on<ImportViaTextPage> {
        input(
            "{ \"relay_overrides\": [ { \"hostname\": \"$relay\", \"ipv4_addr_in\": \"$overrideIp\" } ] }"
        )
        clickImport()
    }
    uiDevice.pressBackThrice()
}

fun ConnectPage.enableWireGuardCustomPort(port: Int) {
    if (port != 51820 && port != 53) {
        error("Port needs to be one of the predefined ports")
    }

    clickSettings()
    on<SettingsPage> { clickVpnSettings() }
    on<VpnSettingsPage> {
        scrollUntilWireGuardCustomPort()
        clickWireguardCustomPort(port)
    }
    uiDevice.pressBackTwice()
}
