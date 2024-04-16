package net.mullvad.mullvadvpn.service

import android.annotation.SuppressLint
import java.io.File
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.asSharedFlow
import net.mullvad.mullvadvpn.lib.endpoint.ApiEndpoint
import net.mullvad.mullvadvpn.lib.endpoint.ApiEndpointConfiguration
import net.mullvad.mullvadvpn.model.AppVersionInfo
import net.mullvad.mullvadvpn.model.CreateCustomListResult
import net.mullvad.mullvadvpn.model.CustomList
import net.mullvad.mullvadvpn.model.Device
import net.mullvad.mullvadvpn.model.DeviceEvent
import net.mullvad.mullvadvpn.model.DeviceListEvent
import net.mullvad.mullvadvpn.model.DeviceState
import net.mullvad.mullvadvpn.model.DnsOptions
import net.mullvad.mullvadvpn.model.GetAccountDataResult
import net.mullvad.mullvadvpn.model.ObfuscationSettings
import net.mullvad.mullvadvpn.model.PlayPurchase
import net.mullvad.mullvadvpn.model.PlayPurchaseInitResult
import net.mullvad.mullvadvpn.model.PlayPurchaseVerifyResult
import net.mullvad.mullvadvpn.model.QuantumResistantState
import net.mullvad.mullvadvpn.model.RelayList
import net.mullvad.mullvadvpn.model.RelayOverride
import net.mullvad.mullvadvpn.model.RelaySettings
import net.mullvad.mullvadvpn.model.RemoveDeviceEvent
import net.mullvad.mullvadvpn.model.RemoveDeviceResult
import net.mullvad.mullvadvpn.model.Settings
import net.mullvad.mullvadvpn.model.SettingsPatchError
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.model.UpdateCustomListResult
import net.mullvad.mullvadvpn.model.VoucherSubmissionResult
import net.mullvad.talpid.util.EventNotifier

@SuppressLint("SdCardPath")
class MullvadDaemon(
    vpnService: MullvadVpnService,
    apiEndpointConfiguration: ApiEndpointConfiguration
) {
    protected var daemonInterfaceAddress = 0L

    val onSettingsChange = EventNotifier<Settings?>(null)
    var onTunnelStateChange = EventNotifier<TunnelState>(TunnelState.Disconnected())

    var onAppVersionInfoChange: ((AppVersionInfo) -> Unit)? = null
    var onRelayListChange: ((RelayList) -> Unit)? = null
    var onDaemonStopped: (() -> Unit)? = null

    private val _deviceStateUpdates = MutableSharedFlow<DeviceState>(extraBufferCapacity = 1)
    val deviceStateUpdates = _deviceStateUpdates.asSharedFlow()

    private val _deviceListUpdates = MutableSharedFlow<DeviceListEvent>(extraBufferCapacity = 1)
    val deviceListUpdates = _deviceListUpdates.asSharedFlow()

    init {
        File("/data/data/net.mullvad.mullvadvpn/rpc-socket").delete()

        System.loadLibrary("mullvad_jni")

        initialize(
            vpnService = vpnService,
            cacheDirectory = vpnService.cacheDir.absolutePath,
            resourceDirectory = vpnService.filesDir.absolutePath,
            apiEndpoint = apiEndpointConfiguration.apiEndpoint()
        )

        //onSettingsChange.notify(getSettings())

        onTunnelStateChange.notify(getState() ?: TunnelState.Disconnected())
    }

    fun connect() {
        connect(daemonInterfaceAddress)
    }

    fun disconnect() {
        disconnect(daemonInterfaceAddress)
    }

    fun getWwwAuthToken(): String {
        return getWwwAuthToken(daemonInterfaceAddress) ?: ""
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

    fun reconnect() {
        reconnect(daemonInterfaceAddress)
    }

    fun getAndEmitDeviceList(accountToken: String): List<Device>? {
        return listDevices(daemonInterfaceAddress, accountToken).also { deviceList ->
            _deviceListUpdates.tryEmit(
                if (deviceList == null) {
                    DeviceListEvent.Error
                } else {
                    DeviceListEvent.Available(accountToken, deviceList)
                }
            )
        }
    }

    fun getAndEmitDeviceState(): DeviceState? {
        return getDevice(daemonInterfaceAddress)?.also { deviceState ->
            _deviceStateUpdates.tryEmit(deviceState)
        }
    }

    fun refreshDevice() {
        updateDevice(daemonInterfaceAddress)
        getAndEmitDeviceState()
    }

    fun removeDevice(accountToken: String, deviceId: String): RemoveDeviceResult {
        return removeDevice(daemonInterfaceAddress, accountToken, deviceId)
    }

    fun shutdown() {
        shutdown(daemonInterfaceAddress)
    }

    fun submitVoucher(voucher: String): VoucherSubmissionResult {
        return submitVoucher(daemonInterfaceAddress, voucher)
    }

    fun initPlayPurchase(): PlayPurchaseInitResult {
        return initPlayPurchase(daemonInterfaceAddress)
    }

    fun verifyPlayPurchase(playPurchase: PlayPurchase): PlayPurchaseVerifyResult {
        return verifyPlayPurchase(daemonInterfaceAddress, playPurchase)
    }

    fun setRelaySettings(update: RelaySettings) {
        setRelaySettings(daemonInterfaceAddress, update)
    }

    fun createCustomList(name: String): CreateCustomListResult =
        createCustomList(daemonInterfaceAddress, name)

    fun deleteCustomList(id: String) {
        deleteCustomList(daemonInterfaceAddress, id)
    }

    fun updateCustomList(customList: CustomList): UpdateCustomListResult =
        updateCustomList(daemonInterfaceAddress, customList)

    fun clearAllRelayOverrides() = clearAllRelayOverrides(daemonInterfaceAddress)

    fun applyJsonSettings(json: String) = applyJsonSettings(daemonInterfaceAddress, json)

    fun exportJsonSettings(): String = exportJsonSettings(daemonInterfaceAddress)

    fun setRelayOverride(relayOverride: RelayOverride) =
        setRelayOverride(daemonInterfaceAddress, relayOverride)

    fun onDestroy() {
        onSettingsChange.unsubscribeAll()
        onTunnelStateChange.unsubscribeAll()

        onAppVersionInfoChange = null
        onRelayListChange = null
        onDaemonStopped = null

        deinitialize()
    }

    private external fun initialize(
        vpnService: MullvadVpnService,
        cacheDirectory: String,
        resourceDirectory: String,
        apiEndpoint: ApiEndpoint?
    )

    private external fun deinitialize()

    private external fun connect(daemonInterfaceAddress: Long)

    private external fun disconnect(daemonInterfaceAddress: Long)

    private external fun getWwwAuthToken(daemonInterfaceAddress: Long): String?

    private external fun getCurrentVersion(daemonInterfaceAddress: Long): String?

    private external fun getRelayLocations(daemonInterfaceAddress: Long): RelayList?

    private external fun getSettings(daemonInterfaceAddress: Long): Settings?

    private external fun getState(daemonInterfaceAddress: Long): TunnelState?

    private external fun reconnect(daemonInterfaceAddress: Long)

    private external fun listDevices(
        daemonInterfaceAddress: Long,
        accountToken: String?
    ): List<Device>?

    // TODO: Review this method when redoing Daemon communication, it can be null which was not
    // considered when this method was initially added.
    private external fun getDevice(daemonInterfaceAddress: Long): DeviceState?

    private external fun updateDevice(daemonInterfaceAddress: Long)

    private external fun removeDevice(
        daemonInterfaceAddress: Long,
        accountToken: String?,
        deviceId: String
    ): RemoveDeviceResult

    private external fun shutdown(daemonInterfaceAddress: Long)

    private external fun submitVoucher(
        daemonInterfaceAddress: Long,
        voucher: String
    ): VoucherSubmissionResult

    private external fun initPlayPurchase(daemonInterfaceAddress: Long): PlayPurchaseInitResult

    private external fun verifyPlayPurchase(
        daemonInterfaceAddress: Long,
        playPurchase: PlayPurchase,
    ): PlayPurchaseVerifyResult

    private external fun setRelaySettings(daemonInterfaceAddress: Long, update: RelaySettings)

    // Used by JNI

    private external fun createCustomList(
        daemonInterfaceAddress: Long,
        name: String
    ): CreateCustomListResult

    private external fun deleteCustomList(daemonInterfaceAddress: Long, id: String)

    private external fun updateCustomList(
        daemonInterfaceAddress: Long,
        customList: CustomList
    ): UpdateCustomListResult

    private external fun clearAllRelayOverrides(daemonInterfaceAddress: Long)

    private external fun applyJsonSettings(
        daemonInterfaceAddress: Long,
        json: String
    ): SettingsPatchError

    private external fun exportJsonSettings(daemonInterfaceAddress: Long): String

    private external fun setRelayOverride(
        daemonInterfaceAddress: Long,
        relayOverride: RelayOverride
    )

    @Suppress("unused")
    private fun notifyAppVersionInfoEvent(appVersionInfo: AppVersionInfo) {
        onAppVersionInfoChange?.invoke(appVersionInfo)
    }

    // Used by JNI
    @Suppress("unused")
    private fun notifyRelayListEvent(relayList: RelayList) {
        onRelayListChange?.invoke(relayList)
    }

    // Used by JNI
    @Suppress("unused")
    private fun notifySettingsEvent(settings: Settings) {
        onSettingsChange.notify(settings)
    }

    // Used by JNI
    @Suppress("unused")
    private fun notifyTunnelStateEvent(event: TunnelState) {
        onTunnelStateChange.notify(event)
    }

    // Used by JNI
    @Suppress("unused")
    private fun notifyDaemonStopped() {
        onDaemonStopped?.invoke()
    }

    // Used by JNI
    @Suppress("unused")
    private fun notifyDeviceEvent(event: DeviceEvent) {
        _deviceStateUpdates.tryEmit(event.newState)
    }

    // Used by JNI
    @Suppress("unused")
    private fun notifyRemoveDeviceEvent(event: RemoveDeviceEvent) {
        _deviceListUpdates.tryEmit(DeviceListEvent.Available(event.accountToken, event.newDevices))
    }
}
