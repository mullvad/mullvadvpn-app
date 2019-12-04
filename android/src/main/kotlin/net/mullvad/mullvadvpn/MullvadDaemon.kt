package net.mullvad.mullvadvpn

import net.mullvad.mullvadvpn.model.AppVersionInfo
import net.mullvad.mullvadvpn.model.GeoIpLocation
import net.mullvad.mullvadvpn.model.GetAccountDataResult
import net.mullvad.mullvadvpn.model.KeygenEvent
import net.mullvad.mullvadvpn.model.PublicKey
import net.mullvad.mullvadvpn.model.RelayList
import net.mullvad.mullvadvpn.model.RelaySettingsUpdate
import net.mullvad.mullvadvpn.model.Settings
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.talpid.util.EventNotifier

class MullvadDaemon(val vpnService: MullvadVpnService) {
    val onSettingsChange = EventNotifier<Settings?>(null)

    var onAppVersionInfoChange: ((AppVersionInfo) -> Unit)? = null
    var onKeygenEvent: ((KeygenEvent) -> Unit)? = null
    var onRelayListChange: ((RelayList) -> Unit)? = null
    var onTunnelStateChange: ((TunnelState) -> Unit)? = null

    init {
        System.loadLibrary("mullvad_jni")
        initialize(vpnService)

        onSettingsChange.notify(getSettings())
    }

    external fun connect()
    external fun disconnect()
    external fun generateWireguardKey(): KeygenEvent?
    external fun getAccountData(accountToken: String): GetAccountDataResult
    external fun getWwwAuthToken(): String
    external fun getCurrentLocation(): GeoIpLocation?
    external fun getCurrentVersion(): String
    external fun getRelayLocations(): RelayList
    external fun getSettings(): Settings
    external fun getState(): TunnelState
    external fun getVersionInfo(): AppVersionInfo?
    external fun getWireguardKey(): PublicKey?
    external fun setAccount(accountToken: String?)
    external fun shutdown()
    external fun updateRelaySettings(update: RelaySettingsUpdate)
    external fun verifyWireguardKey(): Boolean?

    private external fun initialize(vpnService: MullvadVpnService)

    private fun notifyAppVersionInfoEvent(appVersionInfo: AppVersionInfo) {
        onAppVersionInfoChange?.invoke(appVersionInfo)
    }

    private fun notifyKeygenEvent(event: KeygenEvent) {
        onKeygenEvent?.invoke(event)
    }

    private fun notifyRelayListEvent(relayList: RelayList) {
        onRelayListChange?.invoke(relayList)
    }

    private fun notifySettingsEvent(settings: Settings) {
        onSettingsChange.notify(settings)
    }

    private fun notifyTunnelStateEvent(event: TunnelState) {
        onTunnelStateChange?.invoke(event)
    }
}
