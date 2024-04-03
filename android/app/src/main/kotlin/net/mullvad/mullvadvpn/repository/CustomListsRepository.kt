package net.mullvad.mullvadvpn.repository

import arrow.core.Either
import arrow.core.raise.either
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.mapNotNull
import kotlinx.coroutines.withTimeout
import net.mullvad.mullvadvpn.lib.daemon.grpc.ManagementService
import net.mullvad.mullvadvpn.model.CustomList
import net.mullvad.mullvadvpn.model.CustomListId
import net.mullvad.mullvadvpn.model.CustomListName
import net.mullvad.mullvadvpn.model.GeographicLocationConstraint
import net.mullvad.mullvadvpn.model.GetCustomListError
import net.mullvad.mullvadvpn.model.ModifyCustomListError

class CustomListsRepository(
    private val settingsRepository: SettingsRepository,
    private val managementService: ManagementService
) {
    suspend fun createCustomList(name: CustomListName) = managementService.createCustomList(name)

    suspend fun deleteCustomList(id: CustomListId) = managementService.deleteCustomList(id)

    private suspend fun updateCustomList(customList: CustomList) =
        managementService.updateCustomList(customList)

    suspend fun updateCustomListName(
        id: CustomListId,
        name: CustomListName
    ): Either<ModifyCustomListError, Unit> = either {
        val customList = getCustomListById(id).bind()
        updateCustomList(customList.copy(name = name)).bind()
    }

    suspend fun updateCustomListLocations(
        id: CustomListId,
        locations: List<GeographicLocationConstraint>
    ): Either<ModifyCustomListError, Unit> = either {
        val customList = getCustomListById(id).bind()
        updateCustomList(customList.copy(locations = locations)).bind()
    }

    suspend fun getCustomListById(id: CustomListId): Either<GetCustomListError, CustomList> =
        Either.catch {
                withTimeout(GET_CUSTOM_LIST_TIMEOUT_MS) {
                    settingsRepository.settingsUpdates
                        .mapNotNull { settings ->
                            settings?.customLists?.customLists?.find { it.id == id }
                        }
                        .first()
                }
            }
            .mapLeft { GetCustomListError }

    companion object {
        private const val GET_CUSTOM_LIST_TIMEOUT_MS = 5000L
    }
}
