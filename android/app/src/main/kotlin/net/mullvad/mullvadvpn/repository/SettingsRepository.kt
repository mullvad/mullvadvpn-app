package net.mullvad.mullvadvpn.repository

import android.content.SharedPreferences
import android.content.pm.PackageManager
import java.net.InetAddress
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
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
import net.mullvad.mullvadvpn.model.QuantumResistantState
import net.mullvad.mullvadvpn.model.Settings
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.customDns
import net.mullvad.mullvadvpn.ui.serviceconnection.settingsListener
import net.mullvad.mullvadvpn.util.callbackFlowFromNotifier
import net.mullvad.mullvadvpn.util.flatMapReadyConnectionOrDefault

internal const val IS_CONNECT_ON_BOOT_ENABLED_KEY = "is_connect_on_boot_enabled"

class SettingsRepository(
    private val serviceConnectionManager: ServiceConnectionManager,
    private val sharedPreferences: SharedPreferences,
    packageManager: PackageManager,
    dispatcher: CoroutineDispatcher = Dispatchers.IO
) {

    var isConnectOnBootEnabled: MutableStateFlow<Boolean?> =
        MutableStateFlow(
            if (packageManager.hasSystemFeature(PackageManager.FEATURE_LEANBACK))
                sharedPreferences.getBoolean(IS_CONNECT_ON_BOOT_ENABLED_KEY, false)
            else null
        )

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

    fun setWireguardMtu(value: Int?) {
        serviceConnectionManager.settingsListener()?.wireguardMtu = value
    }

    fun setWireguardQuantumResistant(value: QuantumResistantState) {
        serviceConnectionManager.settingsListener()?.wireguardQuantumResistant = value
    }

    fun setObfuscationOptions(value: ObfuscationSettings) {
        serviceConnectionManager.settingsListener()?.obfuscationSettings = value
    }

    fun setAutoConnect(isEnabled: Boolean) {
        serviceConnectionManager.settingsListener()?.autoConnect = isEnabled
    }

    fun setConnectOnBoot(isEnabled: Boolean) {
        sharedPreferences.edit().putBoolean(IS_CONNECT_ON_BOOT_ENABLED_KEY, isEnabled).apply()
        isConnectOnBootEnabled.value = isEnabled
    }

    fun setLocalNetworkSharing(isEnabled: Boolean) {
        serviceConnectionManager.settingsListener()?.allowLan = isEnabled
    }
}
