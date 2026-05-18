package net.mullvad.mullvadvpn.lib.repository

import net.mullvad.mullvadvpn.lib.grpc.ManagementService

class MultihopMigrationRepository(private val managementService: ManagementService) {

    suspend fun getMultihopMigrationState() = managementService.getMigrationEvent()

    suspend fun clearMultihopMigrationState() = managementService.clearMigrationMessage()
}
