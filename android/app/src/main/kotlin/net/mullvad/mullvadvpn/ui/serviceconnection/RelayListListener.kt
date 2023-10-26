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
import net.mullvad.mullvadvpn.model.GeographicLocationConstraint
import net.mullvad.mullvadvpn.model.RelayList
import net.mullvad.mullvadvpn.model.WireguardConstraints

class RelayListListener(
    private val messageHandler: MessageHandler,
    dispatcher: CoroutineDispatcher = Dispatchers.IO
) {
    val relayListEvents: StateFlow<RelayList?> =
        messageHandler
            .events<Event.NewRelayList>()
            .map { it.relayList }
            .stateIn(CoroutineScope(dispatcher), SharingStarted.Eagerly, null)

    fun updateSelectedRelayLocation(value: GeographicLocationConstraint?) {
        messageHandler.trySendRequest(Request.SetRelayLocation(value))
    }

    fun updateSelectedWireguardConstraints(value: WireguardConstraints?) {
        messageHandler.trySendRequest(Request.SetWireguardConstraints(value))
    }
}
