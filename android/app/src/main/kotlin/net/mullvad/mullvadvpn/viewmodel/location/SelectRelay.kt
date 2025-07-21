package net.mullvad.mullvadvpn.viewmodel.location

import arrow.core.Either
import arrow.core.raise.either
import net.mullvad.mullvadvpn.compose.state.RelayListType
import net.mullvad.mullvadvpn.lib.model.Hop
import net.mullvad.mullvadvpn.lib.model.RelayItemId

internal suspend fun selectRelayHop(
    hop: Hop,
    relayListType: RelayListType,
    selectEntryLocation: suspend (RelayItemId) -> Either<Any, Unit>,
    selectExitLocation: suspend (RelayItemId) -> Either<Any, Unit>,
    selectMultihopLocation: suspend (RelayItemId, RelayItemId) -> Either<Any, Unit>,
) =
    either<Any, Unit> {
        when (hop) {
            is Hop.Multi -> {
                val entryConstraint = hop.entry.id
                val exitConstraint = hop.exit.id
                selectMultihopLocation(entryConstraint, exitConstraint)
            }

            is Hop.Single<*> -> {
                val locationConstraint = hop.entry.id
                when (relayListType) {
                    RelayListType.ENTRY -> selectEntryLocation(locationConstraint)
                    RelayListType.EXIT -> selectExitLocation(locationConstraint)
                }
            }
        }
    }
