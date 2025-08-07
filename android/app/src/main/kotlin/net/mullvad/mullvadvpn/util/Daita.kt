package net.mullvad.mullvadvpn.util

import net.mullvad.mullvadvpn.compose.state.MultihopRelayListType
import net.mullvad.mullvadvpn.compose.state.RelayListType

fun shouldFilterByDaita(daitaDirectOnly: Boolean, relayListType: RelayListType) =
    daitaDirectOnly &&
        ((relayListType is RelayListType.Multihop &&
            relayListType.multihopRelayListType == MultihopRelayListType.ENTRY) ||
            relayListType is RelayListType.Single)
