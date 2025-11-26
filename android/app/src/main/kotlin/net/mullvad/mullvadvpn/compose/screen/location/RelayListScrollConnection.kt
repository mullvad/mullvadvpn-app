package net.mullvad.mullvadvpn.compose.screen.location

import kotlinx.coroutines.channels.Channel
import net.mullvad.mullvadvpn.compose.state.RelayListType
import net.mullvad.mullvadvpn.lib.model.RelayItem

typealias ScrollEvent = Pair<RelayListType, RelayItem>

class RelayListScrollConnection {
    val scrollEvents: Channel<ScrollEvent> = Channel()
}
