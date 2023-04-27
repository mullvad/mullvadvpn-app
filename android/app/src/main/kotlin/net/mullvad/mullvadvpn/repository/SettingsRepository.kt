package net.mullvad.mullvadvpn.repository

import java.net.InetAddress
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.onStart
import kotlinx.coroutines.flow.stateIn
import net.mullvad.mullvadvpn.model.CustomDnsOptions
import net.mullvad.mullvadvpn.model.DefaultDnsOptions
import net.mullvad.mullvadvpn.model.DnsOptions
import net.mullvad.mullvadvpn.model.DnsState
import net.mullvad.mullvadvpn.model.ObfuscationSettings
import net.mullvad.mullvadvpn.model.Settings
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.customDns
import net.mullvad.mullvadvpn.ui.serviceconnection.settingsListener
import net.mullvad.mullvadvpn.util.callbackFlowFromNotifier
import net.mullvad.mullvadvpn.util.flatMapReadyConnectionOrDefault

class SettingsRepository(
    private val serviceConnectionManager: ServiceConnectionManager,
    dispatcher: CoroutineDispatcher = Dispatchers.IO
) {
    val settingsUpdates: StateFlow<Settings?> =
        serviceConnectionManager.connectionState
            .flatMapReadyConnectionOrDefault(flowOf()) { state ->
                callbackFlowFromNotifier(state.container.settingsListener.settingsNotifier)
            }
            .onStart { serviceConnectionManager.settingsListener()?.settingsNotifier?.latestEvent }
            .stateIn(CoroutineScope(dispatcher), SharingStarted.WhileSubscribed(), null)

    fun setDnsOptions(
        isCustomDnsEnabled: Boolean,
        dnsList: List<InetAddress>,
        contentBlockersOptions: DefaultDnsOptions
    ) {
        serviceConnectionManager
            .customDns()
            ?.setDnsOptions(
                dnsOptions =
                    DnsOptions(
                        state = if (isCustomDnsEnabled) DnsState.Custom else DnsState.Default,
                        customOptions = CustomDnsOptions(ArrayList(dnsList)),
                        defaultOptions = contentBlockersOptions
                    )
            )
    }

    fun isLocalNetworkSharingEnabled(): Boolean {
        return serviceConnectionManager.settingsListener()?.allowLan ?: false
    }

    fun setWireguardMtu(value: Int?) {
        serviceConnectionManager.settingsListener()?.wireguardMtu = value
    }

    fun setObfuscationOptions(value: ObfuscationSettings) {
        serviceConnectionManager.settingsListener()?.obfuscationSettings = value
    }
}
