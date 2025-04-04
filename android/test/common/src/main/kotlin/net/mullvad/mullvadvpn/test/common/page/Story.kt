package net.mullvad.mullvadvpn.test.common.page

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

fun ConnectPage.enableLocalNetworkSharingStory() {
    clickSettings()
    on<SettingsPage> { clickVpnSettings() }
    on<VpnSettingsPage> { clickLocalNetworkSharingSwitch() }
    uiDevice.pressBackTwice()
}
