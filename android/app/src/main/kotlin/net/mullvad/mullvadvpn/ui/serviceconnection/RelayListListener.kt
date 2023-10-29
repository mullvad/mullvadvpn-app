package net.mullvad.mullvadvpn.ui.serviceconnection

import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import net.mullvad.mullvadvpn.lib.ipc.Event
import net.mullvad.mullvadvpn.lib.ipc.Request
import net.mullvad.mullvadvpn.model.Constraint
import net.mullvad.mullvadvpn.model.GeographicLocationConstraint
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
            .stateIn(CoroutineScope(dispatcher), SharingStarted.Eagerly, defaultRelayList())

    fun updateSelectedRelayLocation(value: GeographicLocationConstraint) {
        messageHandler.trySendRequest(Request.SetRelayLocation(value))
    }

    fun updateSelectedWireguardConstraints(value: WireguardConstraints) {
        messageHandler.trySendRequest(Request.SetWireguardConstraints(value))
    }

    fun updateSelectedOwnershipFilter(value: Constraint<Ownership>) {
        messageHandler.trySendRequest(Request.SetOwnership(value))
    }

    fun updateSelectedProvidersFilter(value: Constraint<Providers>) {
        messageHandler.trySendRequest(Request.SetProviders(value))
    }

    private fun defaultRelayList() = RelayList(ArrayList(), WireguardEndpointData(ArrayList()))
}
