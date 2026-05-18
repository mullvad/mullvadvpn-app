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

            // If user is blocked due to some kind parameter error, and we have a migration state we
            // want to show the error banner
            if (
                tunnelState is TunnelState.Error &&
                    tunnelState.errorState.cause is ErrorStateCause.TunnelParameterError
            ) {
                return@combine InAppNotification.MultihopMigrationBlocked(splitFilterMigration)
            }

            // In the scenario 1A where the user has not enabled Multihop, not enabled DAITA and not
            // set any filters we want to not show the migration in app banner. In all other cases
            // we want to show some kind of in-app banner.
            if (splitFilterMigration.scenario == Scenario.ONE_A) {
                null
            } else {
                InAppNotification.MultihopMigration(splitFilterMigration)
            }
        }
}
