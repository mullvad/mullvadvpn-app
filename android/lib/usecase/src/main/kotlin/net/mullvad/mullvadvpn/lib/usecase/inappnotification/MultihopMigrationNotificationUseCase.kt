package net.mullvad.mullvadvpn.lib.usecase.inappnotification

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import net.mullvad.mullvadvpn.lib.model.ErrorStateCause
import net.mullvad.mullvadvpn.lib.model.InAppNotification
import net.mullvad.mullvadvpn.lib.model.Scenario
import net.mullvad.mullvadvpn.lib.model.TunnelState
import net.mullvad.mullvadvpn.lib.repository.ConnectionProxy
import net.mullvad.mullvadvpn.lib.repository.MultihopMigrationRepository
import net.mullvad.mullvadvpn.lib.repository.UserPreferencesRepository

class MultihopMigrationNotificationUseCase(
    private val multihopMigrationRepository: MultihopMigrationRepository,
    private val connectionProxy: ConnectionProxy,
    private val userPreferencesRepository: UserPreferencesRepository,
) : InAppNotificationUseCase {

    override operator fun invoke(): Flow<InAppNotification?> =
        combine(
            connectionProxy.tunnelState.distinctUntilChanged(),
            multihopMigrationRepository.multihopMigrationState.distinctUntilChanged(),
            userPreferencesRepository.hasSeenMultihopMigrationGuide().distinctUntilChanged(),
        ) { tunnelState, splitFilterMigration, hasSeenMultihopMigrationGuide ->
            if (splitFilterMigration == null) {
                return@combine null
            }

            // User has already seen the migration guide or have dismissed the in-app notification.
            if (hasSeenMultihopMigrationGuide) {
                return@combine null
            }

            // If we do not have a scenario or the scenario is 1A, we do not want to show the
            // migration in app banner.
            // Null is either because not migration has occurred or the migration data has been
            // cleared.
            // Scenario 1A is when the user before updating to a version with the new multihop state
            // did not enable Multihop, did not enable DAITA and did not
            // set any filters.
            if (
                splitFilterMigration.scenario == null ||
                    splitFilterMigration.scenario == Scenario.ONE_A
            ) {
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

            InAppNotification.MultihopMigration(splitFilterMigration)
        }
}
