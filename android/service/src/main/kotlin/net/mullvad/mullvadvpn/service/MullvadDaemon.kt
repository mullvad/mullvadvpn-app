package net.mullvad.mullvadvpn.service

import android.annotation.SuppressLint
import android.net.LocalSocketAddress
import android.util.Log
import com.google.protobuf.Empty
import io.grpc.android.UdsChannelBuilder
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.launch
import mullvad_daemon.management_interface.ManagementServiceGrpcKt
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
import net.mullvad.mullvadvpn.model.LoginResult
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
        System.loadLibrary("mullvad_jni")

        initialize(
            vpnService = vpnService,
            cacheDirectory = vpnService.cacheDir.absolutePath,
            resourceDirectory = vpnService.filesDir.absolutePath,
            apiEndpoint = apiEndpointConfiguration.apiEndpoint()
        )
        val channel =
            UdsChannelBuilder.forPath(
                    "/data/data/net.mullvad.mullvadvpn/rpc-socket",
                    LocalSocketAddress.Namespace.FILESYSTEM
                )
                .build()

        val managementService = ManagementServiceGrpcKt.ManagementServiceCoroutineStub(channel)
        GlobalScope.launch {
            val derp = managementService.getDevice(Empty.getDefaultInstance())

            Log.d("My event", derp.toString())
            managementService.eventsListen(Empty.getDefaultInstance()).collect {
                Log.d("My event", it.toString())
            }
        }

        // val channel: ManagedChannel =
        //
        // NettyChannelBuilder.forAddress(DomainSocketAddress("${vpnService.dataDir}/rpc-socket"))
        //        .channelType(DomainSocketChannel::class.java)
        //        .eventLoopGroup(DefaultEventLoop())
        //        //.eventLoopGroup(EpollEventLoopGroup())
        //        //.channelType(EpollDomainSocketChannel::class.java)
        //        .usePlaintext()
        //        .build()

        onSettingsChange.notify(getSettings())

        onTunnelStateChange.notify(getState() ?: TunnelState.Disconnected())
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

    fun getAccountData(accountToken: String): GetAccountDataResult {
        return getAccountData(daemonInterfaceAddress, accountToken)
    }

    fun getAccountHistory(): String? {
        return getAccountHistory(daemonInterfaceAddress)
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

    fun getVersionInfo(): AppVersionInfo? {
        return getVersionInfo(daemonInterfaceAddress)
    }

    fun reconnect() {
        reconnect(daemonInterfaceAddress)
    }

    fun clearAccountHistory() {
        clearAccountHistory(daemonInterfaceAddress)
    }

    fun loginAccount(accountToken: String): LoginResult {
        return loginAccount(daemonInterfaceAddress, accountToken)
    }

    fun logoutAccount() = logoutAccount(daemonInterfaceAddress)

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

    fun initPlayPurchase(): PlayPurchaseInitResult {
        return initPlayPurchase(daemonInterfaceAddress)
    }

    fun verifyPlayPurchase(playPurchase: PlayPurchase): PlayPurchaseVerifyResult {
        return verifyPlayPurchase(daemonInterfaceAddress, playPurchase)
    }

    fun setRelaySettings(update: RelaySettings) {
        setRelaySettings(daemonInterfaceAddress, update)
    }

    fun setObfuscationSettings(settings: ObfuscationSettings?) {
        setObfuscationSettings(daemonInterfaceAddress, settings)
    }

    fun setQuantumResistant(quantumResistant: QuantumResistantState) {
        setQuantumResistantTunnel(daemonInterfaceAddress, quantumResistant)
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

    private external fun createNewAccount(daemonInterfaceAddress: Long): String?

    private external fun disconnect(daemonInterfaceAddress: Long)

    private external fun getAccountData(
        daemonInterfaceAddress: Long,
        accountToken: String
    ): GetAccountDataResult

    private external fun getAccountHistory(daemonInterfaceAddress: Long): String?

    private external fun getWwwAuthToken(daemonInterfaceAddress: Long): String?

    private external fun getCurrentVersion(daemonInterfaceAddress: Long): String?

    private external fun getRelayLocations(daemonInterfaceAddress: Long): RelayList?

    private external fun getSettings(daemonInterfaceAddress: Long): Settings?

    private external fun getState(daemonInterfaceAddress: Long): TunnelState?

    private external fun getVersionInfo(daemonInterfaceAddress: Long): AppVersionInfo?

    private external fun reconnect(daemonInterfaceAddress: Long)

    private external fun clearAccountHistory(daemonInterfaceAddress: Long)

    private external fun loginAccount(
        daemonInterfaceAddress: Long,
        accountToken: String?
    ): LoginResult

    private external fun logoutAccount(daemonInterfaceAddress: Long)

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

    private external fun setAllowLan(daemonInterfaceAddress: Long, allowLan: Boolean)

    private external fun setAutoConnect(daemonInterfaceAddress: Long, alwaysOn: Boolean)

    private external fun setDnsOptions(daemonInterfaceAddress: Long, dnsOptions: DnsOptions)

    private external fun setWireguardMtu(daemonInterfaceAddress: Long, wireguardMtu: Int?)

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

    private external fun setObfuscationSettings(
        daemonInterfaceAddress: Long,
        settings: ObfuscationSettings?
    )

    private external fun setQuantumResistantTunnel(
        daemonInterfaceAddress: Long,
        quantumResistant: QuantumResistantState
    )

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
