package net.mullvad.mullvadvpn.repository

import java.net.InetAddress
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.shareIn
import net.mullvad.mullvadvpn.model.CustomDnsOptions
import net.mullvad.mullvadvpn.model.DefaultDnsOptions
import net.mullvad.mullvadvpn.model.DnsOptions
import net.mullvad.mullvadvpn.model.DnsState
import net.mullvad.mullvadvpn.ui.serviceconnection.CustomDns
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionState
import net.mullvad.mullvadvpn.ui.serviceconnection.customDns
import net.mullvad.mullvadvpn.ui.serviceconnection.settingsListener

class SettingsRepository(
    private val serviceConnectionManager: ServiceConnectionManager,
    dispatcher: CoroutineDispatcher = Dispatchers.IO
) {
    private val settings = MutableStateFlow(
        MullvadSetting(
            mtu = wireguardMtuString,
            isCustomDnsEnabled = customDns?.isCustomDnsEnabled() ?: false,
            dnsList = customDns?.onDnsServersChanged?.latestEvent ?: emptyList()
        )
    )

    val shared = serviceConnectionManager.connectionState
        .flatMapLatest { state ->
            if (state is ServiceConnectionState.ConnectedReady) {
                flowOf(state.container)
            } else {
                emptyFlow()
            }
        }
        .map {
            it.customDns
        }
        .shareIn(CoroutineScope(dispatcher), SharingStarted.WhileSubscribed())

    fun fetchSettings(): MullvadSetting {
        return settings.value
    }

    fun observeSettings(): Flow<MullvadSetting> = settings

    fun setCustomDnsEnabled(checked: Boolean) {
        var dnsOptions = DnsOptions(
            state = if (checked) DnsState.Custom else DnsState.Default,
            customOptions = CustomDnsOptions(ArrayList(settings.value.dnsList)),
            defaultOptions = DefaultDnsOptions()
        )
        serviceConnectionManager.customDns()?.setDnsOptions(dnsOptions)
//        serviceConnectionManager.customDns()?.onEnabledChanged?.notifyIfChanged(checked)
        settings.value.isCustomDnsEnabled = checked
    }

    fun addDns(address: InetAddress) {
        var newList = ArrayList(settings.value.dnsList)
        newList.add(address)
        var dnsOptions = DnsOptions(
            state = if (settings.value.isCustomDnsEnabled) DnsState.Custom else DnsState.Default,
            customOptions = CustomDnsOptions(newList),
            defaultOptions = DefaultDnsOptions()
        )
        serviceConnectionManager.customDns()?.setDnsOptions(dnsOptions)
    }

    fun removeDns(address: InetAddress) {
        var newList = ArrayList(settings.value.dnsList)
        newList.remove(address)
        var dnsOptions = DnsOptions(
            state = if (settings.value.isCustomDnsEnabled) DnsState.Custom else DnsState.Default,
            customOptions = CustomDnsOptions(newList),
            defaultOptions = DefaultDnsOptions()
        )
        serviceConnectionManager.customDns()?.setDnsOptions(dnsOptions)
    }

    fun editDns(oldAddress: InetAddress, newAddress: InetAddress) {
        var newList = ArrayList(settings.value.dnsList).also {
            it.set(it.indexOf(oldAddress), newAddress)
        }
        var dnsOptions = DnsOptions(
            state = if (settings.value.isCustomDnsEnabled) DnsState.Custom else DnsState.Default,
            customOptions = CustomDnsOptions(newList),
            defaultOptions = DefaultDnsOptions()
        )
        serviceConnectionManager.customDns()?.setDnsOptions(dnsOptions)
    }

    fun isLocalNetworkSharingEnabled(): Boolean {
        return serviceConnectionManager.settingsListener()?.allowLan ?: false
    }

    var wireguardMtu: Int?
        get() = serviceConnectionManager.settingsListener()?.wireguardMtu
        set(value) {
            serviceConnectionManager.settingsListener()?.wireguardMtu = value
            settings.value.mtu = wireguardMtuString
        }

    val wireguardMtuString: String
        get() = wireguardMtu?.let { it.toString() } ?: run { "" }

    val customDns: CustomDns?
        get() = serviceConnectionManager.customDns()
}

data class MullvadSetting(
    var mtu: String,
    var isCustomDnsEnabled: Boolean,
    var dnsList: List<InetAddress>
)
