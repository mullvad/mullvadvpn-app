package net.mullvad.mullvadvpn.usecase.customlists

import arrow.core.Either
import arrow.core.raise.either
import kotlinx.coroutines.flow.firstOrNull
import net.mullvad.mullvadvpn.compose.communication.CustomListAction
import net.mullvad.mullvadvpn.compose.communication.CustomListResult
import net.mullvad.mullvadvpn.lib.model.CreateCustomListError
import net.mullvad.mullvadvpn.lib.model.DeleteCustomListError
import net.mullvad.mullvadvpn.lib.model.GetCustomListError
import net.mullvad.mullvadvpn.lib.model.UpdateCustomListLocationsError
import net.mullvad.mullvadvpn.lib.model.UpdateCustomListNameError
import net.mullvad.mullvadvpn.relaylist.getRelayItemsByCodes
import net.mullvad.mullvadvpn.repository.CustomListsRepository
import net.mullvad.mullvadvpn.repository.RelayListRepository

class CustomListActionUseCase(
    private val customListsRepository: CustomListsRepository,
    private val relayListRepository: RelayListRepository
) {
    suspend fun performAction(
        action: CustomListAction
    ): Either<CustomListActionError, CustomListResult> {
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

    suspend fun performAction(
        action: CustomListAction.Rename
    ): Either<CustomListActionError.Rename, CustomListResult.Renamed> =
        customListsRepository
            .updateCustomListName(action.id, action.newName)
            .map { CustomListResult.Renamed(undo = action.not()) }
            .mapLeft(CustomListActionError::Rename)

    suspend fun performAction(
        action: CustomListAction.Create
    ): Either<CustomListActionError.CreateWithLocations, CustomListResult.Created> = either {
        val customListId =
            customListsRepository
                .createCustomList(action.name)
                .mapLeft(CustomListActionError.CreateWithLocations::Create)
                .bind()

        val locationNames =
            if (action.locations.isNotEmpty()) {
                customListsRepository
                    .updateCustomListLocations(customListId, action.locations)
                    .mapLeft(CustomListActionError.CreateWithLocations::UpdateLocations)
                    .bind()

                relayListRepository.relayList
                    .firstOrNull()
                    ?.getRelayItemsByCodes(action.locations)
                    ?.map { it.name }
                    ?: raise(CustomListActionError.CreateWithLocations.UnableToFetchRelayList)
            } else {
                emptyList()
            }

        CustomListResult.Created(
            id = customListId,
            name = action.name,
            locationNames = locationNames,
            undo = action.not(customListId)
        )
    }

    suspend fun performAction(
        action: CustomListAction.Delete
    ): Either<CustomListActionError.DeleteWithUndo, CustomListResult.Deleted> = either {
        val customList =
            customListsRepository
                .getCustomListById(action.id)
                .mapLeft(CustomListActionError.DeleteWithUndo::Fetch)
                .bind()
        customListsRepository
            .deleteCustomList(action.id)
            .mapLeft(CustomListActionError.DeleteWithUndo::Delete)
            .bind()
        CustomListResult.Deleted(
            undo = action.not(locations = customList.locations, name = customList.name)
        )
    }

    suspend fun performAction(
        action: CustomListAction.UpdateLocations
    ): Either<CustomListActionError.UpdateLocations, CustomListResult.LocationsChanged> = either {
        val customList =
            customListsRepository
                .getCustomListById(action.id)
                .mapLeft(CustomListActionError.UpdateLocations::Fetch)
                .bind()
        customListsRepository
            .updateCustomListLocations(action.id, action.locations)
            .mapLeft(CustomListActionError.UpdateLocations::Update)
            .bind()
        CustomListResult.LocationsChanged(
            name = customList.name,
            undo = action.not(locations = customList.locations)
        )
    }
}

sealed interface CustomListActionError {

    sealed interface CreateWithLocations : CustomListActionError {

        data class Create(val error: CreateCustomListError) :
            CreateWithLocations

        data class UpdateLocations(val error: UpdateCustomListLocationsError) :
            CreateWithLocations

        data object UnableToFetchRelayList : CreateWithLocations
    }

    sealed interface DeleteWithUndo : CustomListActionError {
        data class Fetch(val getCustomListError: GetCustomListError) :
            DeleteWithUndo

        data class Delete(val deleteCustomListError: DeleteCustomListError) :
            DeleteWithUndo
    }

    data class Rename(val error: UpdateCustomListNameError) :
        CustomListActionError

    sealed interface UpdateLocations : CustomListActionError {

        data class Fetch(val getCustomListError: GetCustomListError) :
            UpdateLocations

        data class Update(val updateCustomListLocationsError: UpdateCustomListLocationsError) :
            UpdateLocations
    }
}
