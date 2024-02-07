package net.mullvad.mullvadvpn.ui.serviceconnection

import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.onStart
import kotlinx.coroutines.flow.stateIn
import net.mullvad.mullvadvpn.lib.ipc.Event
import net.mullvad.mullvadvpn.lib.ipc.MessageHandler
import net.mullvad.mullvadvpn.lib.ipc.Request
import net.mullvad.mullvadvpn.lib.ipc.events
import net.mullvad.mullvadvpn.model.Constraint
import net.mullvad.mullvadvpn.model.LocationConstraint
import net.mullvad.mullvadvpn.model.Ownership
import net.mullvad.mullvadvpn.model.Providers
import net.mullvad.mullvadvpn.model.RelayList
import net.mullvad.mullvadvpn.model.WireguardConstraints
import net.mullvad.mullvadvpn.model.WireguardEndpointData

class RelayListListener(
    private val messageHandler: MessageHandler,
    dispatcher: CoroutineDispatcher = Dispatchers.IO
) {
    val relayListEvents: StateFlow<RelayList> =
        messageHandler
            .events<Event.NewRelayList>()
            .map { it.relayList ?: defaultRelayList() }
            // This is added so that we always have a relay list. Otherwise sometimes there would
            // not be a relay list since the fetching of a relay list would be done before the
            // event stream is available.
            .onStart { messageHandler.trySendRequest(Request.FetchRelayList) }
            .stateIn(
                CoroutineScope(dispatcher),
                SharingStarted.WhileSubscribed(),
                defaultRelayList()
            )

    fun updateSelectedRelayLocation(value: LocationConstraint) {
        messageHandler.trySendRequest(Request.SetRelayLocation(value))
    }

    fun updateSelectedWireguardConstraints(value: WireguardConstraints) {
        messageHandler.trySendRequest(Request.SetWireguardConstraints(value))
    }

    fun updateSelectedOwnershipAndProviderFilter(
        ownership: Constraint<Ownership>,
        providers: Constraint<Providers>
    ) {
        messageHandler.trySendRequest(Request.SetOwnershipAndProviders(ownership, providers))
    }

    fun fetchRelayList() {
        messageHandler.trySendRequest(Request.FetchRelayList)
    }

    private fun defaultRelayList() = RelayList(ArrayList(), WireguardEndpointData(ArrayList()))
}
