package net.mullvad.mullvadvpn.repository

import java.net.InetAddress
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.last
import net.mullvad.mullvadvpn.model.CustomDnsOptions
import net.mullvad.mullvadvpn.model.DefaultDnsOptions
import net.mullvad.mullvadvpn.model.DnsOptions
import net.mullvad.mullvadvpn.model.DnsState
import net.mullvad.mullvadvpn.ui.serviceconnection.CustomDns
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.customDns
import net.mullvad.mullvadvpn.ui.serviceconnection.settingsListener

class SettingsRepository(
    private val serviceConnectionManager: ServiceConnectionManager,
    dispatcher: CoroutineDispatcher = Dispatchers.IO
) {
    private val dnsSettings = MutableStateFlow(
        DnsSetting(
            mtu = wireguardMtuString,
            isCustomDnsEnabled = customDns?.isCustomDnsEnabled() ?: false,
            dnsList = customDns?.onDnsServersChanged?.latestEvent ?: emptyList()
        )
    )

//    private val mtuSettings = MutableStateFlow<Int?>(serviceConnectionManager.settingsListener()?.wireguardMtu)
//    val uiState = combine(
//        mtuSettings,
//        dnsSettings
//    ) { mtuSettings, dnsSettings ->
//        val devices = if (deviceList is DeviceList.Available) {
//            deviceList.devices.also { cachedDeviceList = it }
//        } else {
//            cachedDeviceList
//        }
//        val deviceUiItems = devices?.sortedBy { it.creationDate }?.map { device ->
//            DeviceListItemUiState(
//                device,
//                loadingDevices.any { loadingDevice ->
//                    device.id == loadingDevice
//                }
//            )
//        }
//        val isLoading = devices == null
//        val stagedDevice = devices?.firstOrNull { device ->
//            device.id == stagedDeviceId
//        }
//        DeviceListUiState(
//            deviceUiItems = deviceUiItems ?: emptyList(),
//            isLoading = isLoading,
//            stagedDevice = stagedDevice
//        )
//    }.stateIn(CoroutineScope(dispatcher), SharingStarted.WhileSubscribed(), DeviceListUiState.INITIAL)

    suspend fun fetchSettings(): DnsSetting {
        return dnsSettings.last()
    }

    fun observeSettings(): Flow<DnsSetting> = dnsSettings

    fun setDnsOptions(isCustom: Boolean, dnsList: List<InetAddress>) {
        var dnsOptions = DnsOptions(
            state = if (isCustom) DnsState.Custom else DnsState.Default,
            customOptions = CustomDnsOptions(ArrayList(dnsList)),
            defaultOptions = DefaultDnsOptions()
        )
        serviceConnectionManager.customDns()?.setDnsOptions(dnsOptions)
        dnsSettings.value.dnsList = dnsList
        dnsSettings.value.isCustomDnsEnabled = isCustom
    }

    fun isLocalNetworkSharingEnabled(): Boolean {
        return serviceConnectionManager.settingsListener()?.allowLan ?: false
    }

    var wireguardMtu: Int?
        get() = serviceConnectionManager.settingsListener()?.wireguardMtu
        set(value) {
            serviceConnectionManager.settingsListener()?.wireguardMtu = value
            dnsSettings.value.mtu = value?.let { it.toString() } ?: run { "" }
        }

    val wireguardMtuString: String
        get() = wireguardMtu?.let { it.toString() } ?: run { "" }

    val customDns: CustomDns?
        get() = serviceConnectionManager.customDns()
}

data class DnsSetting(
    var mtu: String,
    var isCustomDnsEnabled: Boolean,
    var dnsList: List<InetAddress>
)
