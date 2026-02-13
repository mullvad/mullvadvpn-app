package net.mullvad.mullvadvpn.feature.location.impl

import kotlinx.coroutines.channels.Channel
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.RelayListType

typealias ScrollEvent = Pair<RelayListType, RelayItem>

class RelayListScrollConnection {
    val scrollEvents: Channel<ScrollEvent> = Channel()
}
