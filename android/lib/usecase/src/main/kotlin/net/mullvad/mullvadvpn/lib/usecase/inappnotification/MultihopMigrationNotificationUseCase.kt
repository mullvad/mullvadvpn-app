package net.mullvad.mullvadvpn.lib.usecase.inappnotification

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import net.mullvad.mullvadvpn.lib.model.InAppNotification
import net.mullvad.mullvadvpn.lib.model.MultihopMigrationState
import net.mullvad.mullvadvpn.lib.model.PreviousDaitaState
import net.mullvad.mullvadvpn.lib.repository.MultihopMigrationRepository

class MultihopMigrationNotificationUseCase(
    private val multihopMigrationRepository: MultihopMigrationRepository
) : InAppNotificationUseCase {

    override fun invoke(): Flow<InAppNotification?> =
        multihopMigrationRepository.multihopMigrationState.map { splitFilterMigration ->
            if (splitFilterMigration == null) {
                return@map null
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
