package net.mullvad.mullvadvpn.repository

import java.net.InetAddress
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.stateIn
import net.mullvad.mullvadvpn.lib.daemon.grpc.ManagementService
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.CustomDnsOptions
import net.mullvad.mullvadvpn.lib.model.DefaultDnsOptions
import net.mullvad.mullvadvpn.lib.model.DnsOptions
import net.mullvad.mullvadvpn.lib.model.DnsState
import net.mullvad.mullvadvpn.lib.model.Mtu
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.model.QuantumResistantState
import net.mullvad.mullvadvpn.lib.model.SelectedObfuscation
import net.mullvad.mullvadvpn.lib.model.Settings

class SettingsRepository(
    private val managementService: ManagementService,
    dispatcher: CoroutineDispatcher = Dispatchers.IO,
) {
    val settingsUpdates: StateFlow<Settings?> =
        managementService.settings.stateIn(
            CoroutineScope(dispatcher),
            SharingStarted.WhileSubscribed(),
            null,
        )

    suspend fun setDnsOptions(
        isCustomDnsEnabled: Boolean,
        dnsList: List<InetAddress>,
        contentBlockersOptions: DefaultDnsOptions,
    ) =
        managementService.setDnsOptions(
            DnsOptions(
                state = if (isCustomDnsEnabled) DnsState.Custom else DnsState.Default,
                customOptions = CustomDnsOptions(ArrayList(dnsList)),
                defaultOptions = contentBlockersOptions,
            )
        )

    suspend fun setDnsState(state: DnsState) = managementService.setDnsState(state)

    suspend fun deleteCustomDns(index: Int) = managementService.deleteCustomDns(index)

    suspend fun setCustomDns(index: Int, address: InetAddress) =
        managementService.setCustomDns(index, address)

    suspend fun addCustomDns(address: InetAddress) = managementService.addCustomDns(address)

    suspend fun setCustomObfuscationPort(constraint: Constraint<Port>) =
        managementService.setObfuscationPort(constraint)

    suspend fun setWireguardMtu(mtu: Mtu) = managementService.setWireguardMtu(mtu.value)

    suspend fun resetWireguardMtu() = managementService.resetWireguardMtu()

    suspend fun setWireguardQuantumResistant(value: QuantumResistantState) =
        managementService.setWireguardQuantumResistant(value)

    suspend fun setObfuscation(value: SelectedObfuscation) = managementService.setObfuscation(value)

    suspend fun setAutoConnect(isEnabled: Boolean) = managementService.setAutoConnect(isEnabled)

    suspend fun setLocalNetworkSharing(isEnabled: Boolean) =
        managementService.setAllowLan(isEnabled)

    suspend fun setDaitaEnabled(enabled: Boolean) = managementService.setDaitaEnabled(enabled)
}
