package net.mullvad.mullvadvpn.repository

import java.net.InetAddress
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.onStart
import kotlinx.coroutines.flow.stateIn
import net.mullvad.mullvadvpn.lib.daemon.grpc.ManagementService
import net.mullvad.mullvadvpn.model.CustomDnsOptions
import net.mullvad.mullvadvpn.model.DefaultDnsOptions
import net.mullvad.mullvadvpn.model.DnsOptions
import net.mullvad.mullvadvpn.model.DnsState
import net.mullvad.mullvadvpn.model.ObfuscationSettings
import net.mullvad.mullvadvpn.model.QuantumResistantState
import net.mullvad.mullvadvpn.model.Settings

class SettingsRepository(
    private val managementService: ManagementService,
    dispatcher: CoroutineDispatcher = Dispatchers.IO
) {
    val settingsUpdates: StateFlow<Settings?> =
        managementService.settings
            .onStart { managementService.getSettings() }
            .stateIn(CoroutineScope(dispatcher), SharingStarted.WhileSubscribed(), null)

    suspend fun setDnsOptions(
        isCustomDnsEnabled: Boolean,
        dnsList: List<InetAddress>,
        contentBlockersOptions: DefaultDnsOptions
    ) {
        managementService.setDnsOptions(
            DnsOptions(
                state = if (isCustomDnsEnabled) DnsState.Custom else DnsState.Default,
                customOptions = CustomDnsOptions(ArrayList(dnsList)),
                defaultOptions = contentBlockersOptions
            )
        )
    }

    suspend fun setDnsState(
        state: DnsState,
    ) {
        managementService.setDnsState(state)
    }

    suspend fun deleteCustomDns(address: InetAddress) = managementService.deleteCustomDns(address)

    suspend fun setCustomDns(index: Int, address: InetAddress) =
        managementService.setCustomDns(index, address)

    suspend fun setWireguardMtu(value: Int?) = managementService.setWireguardMtu(value ?: 0)

    suspend fun setWireguardQuantumResistant(value: QuantumResistantState) =
        managementService.setWireguardQuantumResistant(value)

    suspend fun setObfuscationOptions(value: ObfuscationSettings) =
        managementService.setObfuscationOptions(value)

    suspend fun setAutoConnect(isEnabled: Boolean) = managementService.setAutoConnect(isEnabled)

    suspend fun setLocalNetworkSharing(isEnabled: Boolean) =
        managementService.setAllowLan(isEnabled)
}
