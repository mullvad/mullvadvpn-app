package net.mullvad.mullvadvpn.util

import net.mullvad.mullvadvpn.compose.state.RelayListType

fun showOnlyRelaysWithDaita(
    isDaitaEnabled: Boolean,
    isMultihopEnabled: Boolean,
    relayListType: RelayListType,
) =
    isDaitaEnabled &&
        (relayListType == RelayListType.ENTRY ||
            !isMultihopEnabled && relayListType == RelayListType.EXIT)
