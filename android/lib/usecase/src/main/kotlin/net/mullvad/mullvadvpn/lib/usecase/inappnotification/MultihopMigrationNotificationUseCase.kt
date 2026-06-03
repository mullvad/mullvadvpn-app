package net.mullvad.mullvadvpn.lib.usecase.inappnotification

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
import net.mullvad.mullvadvpn.lib.model.ErrorStateCause
import net.mullvad.mullvadvpn.lib.model.InAppNotification
import net.mullvad.mullvadvpn.lib.model.MultihopMigrationState
import net.mullvad.mullvadvpn.lib.model.PreviousDaitaState
import net.mullvad.mullvadvpn.lib.model.TunnelState
import net.mullvad.mullvadvpn.lib.repository.ConnectionProxy
import net.mullvad.mullvadvpn.lib.repository.MultihopMigrationRepository

class MultihopMigrationNotificationUseCase(
    private val multihopMigrationRepository: MultihopMigrationRepository,
    private val connectionProxy: ConnectionProxy,
) : InAppNotificationUseCase {

    override operator fun invoke(): Flow<InAppNotification?> =
        combine(connectionProxy.tunnelState, multihopMigrationRepository.multihopMigrationState) {
            tunnelState,
            splitFilterMigration ->
            if (splitFilterMigration == null) {
                return@combine null
            }

            // If user is blocked due to some kind parameter error, and we have a migration state we
            // want to show the error banner
            if (
                tunnelState is TunnelState.Error &&
                    tunnelState.errorState.cause is ErrorStateCause.TunnelParameterError
            ) {
                return@combine InAppNotification.MultihopMigrationBlocked(splitFilterMigration)
            }

            // In the scenario where the user has not enabled Multihop, not enabled DAITA and not
            // set any filters we want to not show the migration in app banner. In all other cases
            // we want to show some kind of in-app banner.
            if (
                splitFilterMigration.multihopMigrationState ==
                    MultihopMigrationState.OFF_TO_WHEN_NEEDED &&
                    !splitFilterMigration.filtersSet &&
                    splitFilterMigration.daitaMigration == PreviousDaitaState.OFF
            ) {
                null
            } else {
                InAppNotification.MultihopMigration(splitFilterMigration)
            }
        }
}
