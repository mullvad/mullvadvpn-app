package net.mullvad.mullvadvpn.util

import net.mullvad.mullvadvpn.compose.state.RelayListType

fun shouldFilterByDaita(
    daitaDirectOnly: Boolean,
    isMultihopEnabled: Boolean,
    relayListType: RelayListType,
) =
    daitaDirectOnly &&
        (relayListType == RelayListType.ENTRY ||
            !isMultihopEnabled && relayListType == RelayListType.EXIT)
