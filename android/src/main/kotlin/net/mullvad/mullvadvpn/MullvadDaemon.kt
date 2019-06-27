package net.mullvad.mullvadvpn

import net.mullvad.mullvadvpn.model.AccountData
import net.mullvad.mullvadvpn.model.GeoIpLocation
import net.mullvad.mullvadvpn.model.PublicKey
import net.mullvad.mullvadvpn.model.RelayList
import net.mullvad.mullvadvpn.model.RelaySettingsUpdate
import net.mullvad.mullvadvpn.model.Settings
import net.mullvad.mullvadvpn.model.TunnelState

class MullvadDaemon(val vpnService: MullvadVpnService) {
    init {
        System.loadLibrary("mullvad_jni")
        initialize(vpnService)
    }

    var onRelayListChange: ((RelayList) -> Unit)? = null
    var onSettingsChange: ((Settings) -> Unit)? = null
    var onTunnelStateChange: ((TunnelState) -> Unit)? = null

    external fun connect()
    external fun disconnect()
    external fun generateWireguardKey(): Boolean
    external fun getAccountData(accountToken: String): AccountData?
    external fun getCurrentLocation(): GeoIpLocation?
    external fun getRelayLocations(): RelayList
    external fun getSettings(): Settings
    external fun getState(): TunnelState
    external fun getWireguardKey(): PublicKey?
    external fun setAccount(accountToken: String?)
    external fun updateRelaySettings(update: RelaySettingsUpdate)

    private external fun initialize(vpnService: MullvadVpnService)

    private fun notifyRelayListEvent(relayList: RelayList) {
        onRelayListChange?.invoke(relayList)
    }

    private fun notifySettingsEvent(settings: Settings) {
        onSettingsChange?.invoke(settings)
    }

    private fun notifyTunnelStateEvent(event: TunnelState) {
        onTunnelStateChange?.invoke(event)
    }
}
