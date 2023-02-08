package net.mullvad.mullvadvpn.repository

import net.mullvad.mullvadvpn.model.DnsOptions
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.customDns

class SettingsRepository(
    private val serviceConnectionManager: ServiceConnectionManager
) {
    fun setDnsOptions(dnsOptions: DnsOptions) {
        serviceConnectionManager.customDns()?.setDnsOptions(dnsOptions)
    }
}
