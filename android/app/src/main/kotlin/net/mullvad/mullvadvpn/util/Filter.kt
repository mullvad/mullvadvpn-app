package net.mullvad.mullvadvpn.util

import net.mullvad.mullvadvpn.compose.state.MultihopRelayListType
import net.mullvad.mullvadvpn.compose.state.RelayListType

fun shouldFilterByDaita(daitaDirectOnly: Boolean, relayListType: RelayListType) =
    when (relayListType) {
        RelayListType.Single -> daitaDirectOnly
        is RelayListType.Multihop ->
            daitaDirectOnly && relayListType.multihopRelayListType == MultihopRelayListType.ENTRY
    }

fun shouldFilterByQuic(isQuicEnabled: Boolean, relayListType: RelayListType) =
    when (relayListType) {
        RelayListType.Single -> isQuicEnabled
        is RelayListType.Multihop ->
            isQuicEnabled && relayListType.multihopRelayListType == MultihopRelayListType.ENTRY
    }
