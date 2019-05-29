package net.mullvad.mullvadvpn

import net.mullvad.mullvadvpn.model.AccountData
import net.mullvad.mullvadvpn.model.PublicKey
import net.mullvad.mullvadvpn.model.RelayList
import net.mullvad.mullvadvpn.model.RelaySettingsUpdate
import net.mullvad.mullvadvpn.model.Settings
import net.mullvad.mullvadvpn.model.TunnelStateTransition

class MullvadDaemon(val vpnService: MullvadVpnService) {
    init {
        System.loadLibrary("mullvad_jni")
        initialize(vpnService)
    }

    var onTunnelStateChange: ((TunnelStateTransition) -> Unit)? = null

    external fun connect()
    external fun disconnect()
    external fun generateWireguardKey(): Boolean
    external fun getAccountData(accountToken: String): AccountData?
    external fun getRelayLocations(): RelayList
    external fun getSettings(): Settings
    external fun getWireguardKey(): PublicKey?
    external fun setAccount(accountToken: String?)
    external fun updateRelaySettings(update: RelaySettingsUpdate)

    private external fun initialize(vpnService: MullvadVpnService)

    private fun notifyTunnelStateEvent(event: TunnelStateTransition) {
        onTunnelStateChange?.invoke(event)
    }
}
