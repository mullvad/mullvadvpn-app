package net.mullvad.mullvadvpn.lib.usecase

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
import net.mullvad.mullvadvpn.lib.model.Scenario
import net.mullvad.mullvadvpn.lib.repository.MultihopMigrationRepository
import net.mullvad.mullvadvpn.lib.repository.UserPreferencesRepository

class MultihopGuideMigrationHintUseCase(
    private val userPreferencesRepository: UserPreferencesRepository,
    private val migrationRepository: MultihopMigrationRepository,
) {
    operator fun invoke(): Flow<Boolean> =
        combine(
            userPreferencesRepository.hasSeenMultihopMigrationGuide(),
            migrationRepository.multihopMigrationState,
        ) { hasSeenGuide, migrationState ->
            migrationState != null &&
                migrationState.scenario != null &&
                migrationState.scenario != Scenario.ONE_A &&
                !hasSeenGuide
        }
}
