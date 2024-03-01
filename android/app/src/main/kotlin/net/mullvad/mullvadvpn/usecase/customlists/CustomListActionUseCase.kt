package net.mullvad.mullvadvpn.usecase.customlists

import kotlinx.coroutines.flow.firstOrNull
import net.mullvad.mullvadvpn.compose.communication.CustomListAction
import net.mullvad.mullvadvpn.compose.communication.CustomListResult
import net.mullvad.mullvadvpn.model.CreateCustomListResult
import net.mullvad.mullvadvpn.model.CustomList
import net.mullvad.mullvadvpn.model.GeographicLocationConstraint
import net.mullvad.mullvadvpn.model.UpdateCustomListResult
import net.mullvad.mullvadvpn.relaylist.getRelayItemsByCodes
import net.mullvad.mullvadvpn.repository.CustomListsRepository
import net.mullvad.mullvadvpn.usecase.RelayListUseCase

class CustomListActionUseCase(
    private val customListsRepository: CustomListsRepository,
    private val relayListUseCase: RelayListUseCase
) {
    suspend fun performAction(action: CustomListAction): Result<CustomListResult> {
        return when (action) {
            is CustomListAction.Create -> {
                performAction(action)
            }
            is CustomListAction.Rename -> {
                performAction(action)
            }
            is CustomListAction.Delete -> {
                performAction(action)
            }
            is CustomListAction.UpdateLocations -> {
                performAction(action)
            }
        }
    }

    suspend fun performAction(action: CustomListAction.Rename): Result<CustomListResult.Renamed> =
        when (
            val result =
                customListsRepository.updateCustomListName(action.customListId, action.newName)
        ) {
            is UpdateCustomListResult.Ok ->
                Result.success(CustomListResult.Renamed(undo = action.not()))
            is UpdateCustomListResult.Error -> Result.failure(CustomListsException(result.error))
        }

    suspend fun performAction(action: CustomListAction.Create): Result<CustomListResult.Created> =
        when (val result = customListsRepository.createCustomList(action.name)) {
            is CreateCustomListResult.Ok -> {
                if (action.locations.isNotEmpty()) {
                    customListsRepository.updateCustomListLocationsFromCodes(
                        result.id,
                        action.locations
                    )
                    val locationNames =
                        relayListUseCase
                            .relayList()
                            .firstOrNull()
                            ?.getRelayItemsByCodes(action.locations)
                            ?.map { it.name }
                    Result.success(
                        CustomListResult.Created(
                            id = result.id,
                            name = action.name,
                            locationName = locationNames?.first(),
                            undo = action.not(result.id)
                        )
                    )
                } else {
                    Result.success(
                        CustomListResult.Created(
                            id = result.id,
                            name = action.name,
                            locationName = null,
                            undo = action.not(result.id)
                        )
                    )
                }
            }
            is CreateCustomListResult.Error -> Result.failure(CustomListsException(result.error))
        }

    fun performAction(action: CustomListAction.Delete): Result<CustomListResult.Deleted> {
        val customList: CustomList? = customListsRepository.getCustomListById(action.customListId)
        val oldLocations = customList.locations()
        val name = customList?.name ?: ""
        customListsRepository.deleteCustomList(action.customListId)
        return Result.success(
            CustomListResult.Deleted(undo = action.not(locations = oldLocations, name = name))
        )
    }

    suspend fun performAction(
        action: CustomListAction.UpdateLocations
    ): Result<CustomListResult.LocationsChanged> {
        val customList: CustomList? = customListsRepository.getCustomListById(action.customListId)
        val oldLocations = customList.locations()
        val name = customList?.name ?: ""
        customListsRepository.updateCustomListLocationsFromCodes(
            action.customListId,
            action.locations
        )
        return Result.success(
            CustomListResult.LocationsChanged(
                name = name,
                undo = action.not(locations = oldLocations)
            )
        )
    }

    private fun CustomList?.locations(): List<String> =
        this?.locations?.map {
            when (it) {
                is GeographicLocationConstraint.City -> it.cityCode
                is GeographicLocationConstraint.Country -> it.countryCode
                is GeographicLocationConstraint.Hostname -> it.hostname
            }
        } ?: emptyList()
}
