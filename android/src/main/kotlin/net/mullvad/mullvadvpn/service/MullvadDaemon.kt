package net.mullvad.mullvadvpn.service

import net.mullvad.mullvadvpn.model.AppVersionInfo
import net.mullvad.mullvadvpn.model.DnsOptions
import net.mullvad.mullvadvpn.model.GeoIpLocation
import net.mullvad.mullvadvpn.model.GetAccountDataResult
import net.mullvad.mullvadvpn.model.KeygenEvent
import net.mullvad.mullvadvpn.model.PublicKey
import net.mullvad.mullvadvpn.model.RelayList
import net.mullvad.mullvadvpn.model.RelaySettingsUpdate
import net.mullvad.mullvadvpn.model.Settings
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.model.VoucherSubmissionResult
import net.mullvad.talpid.util.EventNotifier

class MullvadDaemon(val vpnService: MullvadVpnService) {
    protected var daemonInterfaceAddress = 0L

    val onSettingsChange = EventNotifier<Settings?>(null)

    var onAppVersionInfoChange: ((AppVersionInfo) -> Unit)? = null
    var onKeygenEvent: ((KeygenEvent) -> Unit)? = null
    var onRelayListChange: ((RelayList) -> Unit)? = null
    var onTunnelStateChange: ((TunnelState) -> Unit)? = null
    var onDaemonStopped: (() -> Unit)? = null

    init {
        System.loadLibrary("mullvad_jni")
        initialize(vpnService, vpnService.cacheDir.absolutePath, vpnService.filesDir.absolutePath)

        onSettingsChange.notify(getSettings())
    }

    fun connect() {
        connect(daemonInterfaceAddress)
    }

    fun createNewAccount(): String? {
        return createNewAccount(daemonInterfaceAddress)
    }

    fun disconnect() {
        disconnect(daemonInterfaceAddress)
    }

    fun generateWireguardKey(): KeygenEvent? {
        return generateWireguardKey(daemonInterfaceAddress)
    }

    fun getAccountData(accountToken: String): GetAccountDataResult {
        return getAccountData(daemonInterfaceAddress, accountToken)
    }

    fun getAccountHistory(): ArrayList<String>? {
        return getAccountHistory(daemonInterfaceAddress)
    }

    fun getWwwAuthToken(): String {
        return getWwwAuthToken(daemonInterfaceAddress) ?: ""
    }

    fun getCurrentLocation(): GeoIpLocation? {
        return getCurrentLocation(daemonInterfaceAddress)
    }

    fun getCurrentVersion(): String? {
        return getCurrentVersion(daemonInterfaceAddress)
    }

    fun getRelayLocations(): RelayList? {
        return getRelayLocations(daemonInterfaceAddress)
    }

    fun getSettings(): Settings? {
        return getSettings(daemonInterfaceAddress)
    }

    fun getState(): TunnelState? {
        return getState(daemonInterfaceAddress)
    }

    fun getVersionInfo(): AppVersionInfo? {
        return getVersionInfo(daemonInterfaceAddress)
    }

    fun getWireguardKey(): PublicKey? {
        return getWireguardKey(daemonInterfaceAddress)
    }

    fun reconnect() {
        reconnect(daemonInterfaceAddress)
    }

    fun removeAccountFromHistory(accountToken: String) {
        removeAccountFromHistory(daemonInterfaceAddress, accountToken)
    }

    fun setAccount(accountToken: String?) {
        setAccount(daemonInterfaceAddress, accountToken)
    }

    fun setAllowLan(allowLan: Boolean) {
        setAllowLan(daemonInterfaceAddress, allowLan)
    }

    fun setAutoConnect(autoConnect: Boolean) {
        setAutoConnect(daemonInterfaceAddress, autoConnect)
    }

    fun setDnsOptions(dnsOptions: DnsOptions) {
        setDnsOptions(daemonInterfaceAddress, dnsOptions)
    }

    fun setWireguardMtu(wireguardMtu: Int?) {
        setWireguardMtu(daemonInterfaceAddress, wireguardMtu)
    }

    fun shutdown() {
        shutdown(daemonInterfaceAddress)
    }

    fun submitVoucher(voucher: String): VoucherSubmissionResult {
        return submitVoucher(daemonInterfaceAddress, voucher)
    }

    fun updateRelaySettings(update: RelaySettingsUpdate) {
        updateRelaySettings(daemonInterfaceAddress, update)
    }

    fun verifyWireguardKey(): Boolean? {
        return verifyWireguardKey(daemonInterfaceAddress)
    }

    fun onDestroy() {
        onSettingsChange.unsubscribeAll()

        onAppVersionInfoChange = null
        onKeygenEvent = null
        onRelayListChange = null
        onTunnelStateChange = null
        onDaemonStopped = null

        deinitialize()
    }

    private external fun initialize(
        vpnService: MullvadVpnService,
        cacheDirectory: String,
        resourceDirectory: String
    )
    private external fun deinitialize()

    private external fun connect(daemonInterfaceAddress: Long)
    private external fun createNewAccount(daemonInterfaceAddress: Long): String?
    private external fun disconnect(daemonInterfaceAddress: Long)
    private external fun generateWireguardKey(daemonInterfaceAddress: Long): KeygenEvent?
    private external fun getAccountData(
        daemonInterfaceAddress: Long,
        accountToken: String
    ): GetAccountDataResult
    private external fun getAccountHistory(daemonInterfaceAddress: Long): ArrayList<String>?
    private external fun getWwwAuthToken(daemonInterfaceAddress: Long): String?
    private external fun getCurrentLocation(daemonInterfaceAddress: Long): GeoIpLocation?
    private external fun getCurrentVersion(daemonInterfaceAddress: Long): String?
    private external fun getRelayLocations(daemonInterfaceAddress: Long): RelayList?
    private external fun getSettings(daemonInterfaceAddress: Long): Settings?
    private external fun getState(daemonInterfaceAddress: Long): TunnelState?
    private external fun getVersionInfo(daemonInterfaceAddress: Long): AppVersionInfo?
    private external fun getWireguardKey(daemonInterfaceAddress: Long): PublicKey?
    private external fun reconnect(daemonInterfaceAddress: Long)
    private external fun removeAccountFromHistory(
        daemonInterfaceAddress: Long,
        accountToken: String
    )
    private external fun setAccount(daemonInterfaceAddress: Long, accountToken: String?)
    private external fun setAllowLan(daemonInterfaceAddress: Long, allowLan: Boolean)
    private external fun setAutoConnect(daemonInterfaceAddress: Long, alwaysOn: Boolean)
    private external fun setDnsOptions(daemonInterfaceAddress: Long, dnsOptions: DnsOptions)
    private external fun setWireguardMtu(daemonInterfaceAddress: Long, wireguardMtu: Int?)
    private external fun shutdown(daemonInterfaceAddress: Long)
    private external fun submitVoucher(
        daemonInterfaceAddress: Long,
        voucher: String
    ): VoucherSubmissionResult
    private external fun updateRelaySettings(
        daemonInterfaceAddress: Long,
        update: RelaySettingsUpdate
    )
    private external fun verifyWireguardKey(daemonInterfaceAddress: Long): Boolean?

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

    private fun notifyDaemonStopped() {
        onDaemonStopped?.invoke()
    }
}
