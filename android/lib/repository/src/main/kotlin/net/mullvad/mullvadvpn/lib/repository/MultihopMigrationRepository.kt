package net.mullvad.mullvadvpn.lib.repository

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.onSubscription
import net.mullvad.mullvadvpn.lib.grpc.ManagementService
import net.mullvad.mullvadvpn.lib.model.SplitFilterMigration

class MultihopMigrationRepository(private val managementService: ManagementService) {

    private val _multihopMigrationState = MutableStateFlow<SplitFilterMigration?>(null)
    val multihopMigrationState: Flow<SplitFilterMigration?> =
        _multihopMigrationState.onSubscription {
            _multihopMigrationState.value = getMultihopMigrationState().getOrNull()
        }

    suspend fun getMultihopMigrationState() = managementService.getMigrationEvent()

    suspend fun clearMultihopMigrationState() =
        managementService.clearMigrationMessage().onRight {
            _multihopMigrationState.value = getMultihopMigrationState().getOrNull()
        }
}
