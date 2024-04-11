package net.mullvad.mullvadvpn.repository

import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.stateIn
import net.mullvad.mullvadvpn.lib.daemon.grpc.ManagementService
import net.mullvad.mullvadvpn.model.RelayItem
import net.mullvad.mullvadvpn.model.WireguardConstraints
import net.mullvad.mullvadvpn.model.WireguardEndpointData

class RelayListRepository(
    private val managementService: ManagementService,
    dispatcher: CoroutineDispatcher = Dispatchers.IO
) {
    val relayList: StateFlow<List<RelayItem.Location.Country>> =
        managementService.relayCountries.stateIn(
            CoroutineScope(dispatcher),
            SharingStarted.WhileSubscribed(),
            emptyList()
        )

    val wireguardEndpointData: StateFlow<WireguardEndpointData> =
        managementService.wireguardEndpointData.stateIn(
            CoroutineScope(dispatcher),
            SharingStarted.WhileSubscribed(),
            defaultWireguardEndpointData()
        )

    suspend fun updateSelectedWireguardConstraints(value: WireguardConstraints) =
        managementService.setWireguardConstraints(value)

    private fun defaultWireguardEndpointData() = WireguardEndpointData(emptyList())
}
