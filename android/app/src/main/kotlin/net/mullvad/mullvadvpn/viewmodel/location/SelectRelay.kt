package net.mullvad.mullvadvpn.viewmodel.location

import arrow.core.Either
import arrow.core.raise.either
import net.mullvad.mullvadvpn.compose.state.RelayListSelection
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.RelayItemId

internal suspend fun selectRelayItem(
    relayItem: RelayItem,
    relayListSelection: RelayListSelection,
    selectEntryLocation: suspend (RelayItemId) -> Either<Any, Unit>,
    selectExitLocation: suspend (RelayItemId) -> Either<Any, Unit>,
) =
    either<Any, Unit> {
        val locationConstraint = relayItem.id
        when (relayListSelection) {
            RelayListSelection.Entry -> selectEntryLocation(locationConstraint)
            RelayListSelection.Exit -> selectExitLocation(locationConstraint)
        }
    }
