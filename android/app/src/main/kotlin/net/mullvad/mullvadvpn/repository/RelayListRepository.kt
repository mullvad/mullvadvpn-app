package net.mullvad.mullvadvpn.repository

import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import net.mullvad.mullvadvpn.lib.daemon.grpc.ManagementService
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.PortRange
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.RelayItemId
import net.mullvad.mullvadvpn.lib.model.WireguardConstraints
import net.mullvad.mullvadvpn.lib.model.WireguardEndpointData
import net.mullvad.mullvadvpn.relaylist.findByGeoLocationId

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

    val selectedLocation: StateFlow<Constraint<RelayItemId>> =
        managementService.settings
            .map { it.relaySettings.relayConstraints.location }
            .stateIn(CoroutineScope(dispatcher), SharingStarted.WhileSubscribed(), Constraint.Any)

    val portRanges: Flow<List<PortRange>> =
        wireguardEndpointData.map { it.portRanges }.distinctUntilChanged()

    suspend fun updateSelectedRelayLocation(value: RelayItemId) =
        managementService.setRelayLocation(value)

    suspend fun updateSelectedWireguardConstraints(value: WireguardConstraints) =
        managementService.setWireguardConstraints(value)

    fun find(geoLocationId: GeoLocationId) = relayList.value.findByGeoLocationId(geoLocationId)

    private fun defaultWireguardEndpointData() = WireguardEndpointData(emptyList())
}
