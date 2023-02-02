package net.mullvad.mullvadvpn.repository

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import net.mullvad.mullvadvpn.ui.serviceconnection.CustomDns
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.customDns
import net.mullvad.mullvadvpn.ui.serviceconnection.settingsListener
import net.mullvad.mullvadvpn.model.DnsOptions
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.customDns

class SettingsRepository(
    private val serviceConnectionManager: ServiceConnectionManager
) {
    private val settings = MutableStateFlow<MullvadSetting>(
        MullvadSetting(
            mtu = wireguardMtuString,
            isCustomDnsEnabled = customDns?.isCustomDnsEnabled() ?: false,
            dnsList = emptyList()
        )
    )

    fun fetchSettings(): MullvadSetting {
        return settings.value
    }

    fun observeSettings(): Flow<MullvadSetting> = settings

    fun setCustomDnsEnabled(checked: Boolean) {
        if (checked) {
            serviceConnectionManager.customDns()?.enable()
        } else {
            serviceConnectionManager.customDns()?.disable()
        }
        serviceConnectionManager.customDns()?.onEnabledChanged?.notifyIfChanged(checked)
        settings.value.isCustomDnsEnabled = checked
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
    var dnsList: List<String>
)
    fun setDnsOptions(dnsOptions: DnsOptions) {
        serviceConnectionManager.customDns()?.setDnsOptions(dnsOptions)
    }
}
