package net.mullvad.mullvadvpn.app.service.migration

import kotlinx.coroutines.flow.firstOrNull
import net.mullvad.mullvadvpn.lib.common.util.entryLocation
import net.mullvad.mullvadvpn.lib.grpc.ManagementService
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.repository.UserPreferencesRepository

class MigrateEntryLocation(
    private val managementService: ManagementService,
    private val userPreferencesRepository: UserPreferencesRepository,
) {
    suspend fun migrate() {
        if (!userPreferencesRepository.hasMigratedEntryLocation()) {
            if (
                managementService.settings.firstOrNull()?.entryLocation() !=
                    Constraint.Only(GeoLocationId.Country(""))
            ) {
                managementService.setEntryLocation(Constraint.Any)
            }
            userPreferencesRepository.setHasMigratedEntryLocation(true)
        }
    }
}
