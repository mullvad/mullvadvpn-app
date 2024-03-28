package net.mullvad.mullvadvpn.ui.serviceconnection

import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.flow.stateIn
import net.mullvad.mullvadvpn.lib.daemon.grpc.ManagementService
import net.mullvad.mullvadvpn.model.Constraint
import net.mullvad.mullvadvpn.model.LocationConstraint
import net.mullvad.mullvadvpn.model.Ownership
import net.mullvad.mullvadvpn.model.Providers
import net.mullvad.mullvadvpn.model.RelayList
import net.mullvad.mullvadvpn.model.WireguardConstraints
import net.mullvad.mullvadvpn.model.WireguardEndpointData

class RelayListListener(
    private val managementService: ManagementService,
    dispatcher: CoroutineDispatcher = Dispatchers.IO
) {
    val relayListEvents: StateFlow<RelayList> =
        emptyFlow<RelayList>()
            .stateIn(
                CoroutineScope(dispatcher),
                SharingStarted.WhileSubscribed(),
                defaultRelayList()
            )

    suspend fun updateSelectedRelayLocation(value: LocationConstraint) {
        managementService.setRelayLocation(value)
    }

    fun updateSelectedWireguardConstraints(value: WireguardConstraints) {
        //        messageHandler.trySendRequest(Request.SetWireguardConstraints(value))
    }

    fun updateSelectedOwnershipAndProviderFilter(
        ownership: Constraint<Ownership>,
        providers: Constraint<Providers>
    ) {
        //        messageHandler.trySendRequest(Request.SetOwnershipAndProviders(ownership,
        // providers))
    }

    fun fetchRelayList() {
        //        messageHandler.trySendRequest(Request.FetchRelayList)
    }

    private fun defaultRelayList() = RelayList(ArrayList(), WireguardEndpointData(ArrayList()))
}
