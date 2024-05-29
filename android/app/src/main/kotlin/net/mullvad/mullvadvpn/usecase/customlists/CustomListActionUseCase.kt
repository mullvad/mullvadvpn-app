package net.mullvad.mullvadvpn.usecase.customlists

import arrow.core.Either
import arrow.core.raise.either
import kotlinx.coroutines.flow.firstOrNull
import net.mullvad.mullvadvpn.compose.communication.Created
import net.mullvad.mullvadvpn.compose.communication.CustomListAction
import net.mullvad.mullvadvpn.compose.communication.CustomListSuccess
import net.mullvad.mullvadvpn.compose.communication.Deleted
import net.mullvad.mullvadvpn.compose.communication.LocationsChanged
import net.mullvad.mullvadvpn.compose.communication.Renamed
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
    ): Either<CustomListActionError, CustomListSuccess> {
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

    suspend fun performAction(action: CustomListAction.Rename): Either<RenameError, Renamed> =
        customListsRepository
            .updateCustomListName(action.id, action.newName)
            .map { Renamed(undo = action.not()) }
            .mapLeft(::RenameError)

    suspend fun performAction(
        action: CustomListAction.Create
    ): Either<CreateWithLocationsError, Created> = either {
        val customListId =
            customListsRepository
                .createCustomList(action.name)
                .mapLeft(CreateWithLocationsError::CreateActionError)
                .bind()

        val locationNames =
            if (action.locations.isNotEmpty()) {
                customListsRepository
                    .updateCustomListLocations(customListId, action.locations)
                    .mapLeft(CreateWithLocationsError::UpdateLocationsActionError)
                    .bind()

                relayListRepository.relayList
                    .firstOrNull()
                    ?.getRelayItemsByCodes(action.locations)
                    ?.map { it.name } ?: raise(CreateWithLocationsError.UnableToFetchRelayList)
            } else {
                emptyList()
            }

        Created(
            id = customListId,
            name = action.name,
            locationNames = locationNames,
            undo = action.not(customListId)
        )
    }

    suspend fun performAction(
        action: CustomListAction.Delete
    ): Either<DeleteWithUndoError, Deleted> = either {
        val customList =
            customListsRepository
                .getCustomListById(action.id)
                .mapLeft(DeleteWithUndoError::Fetch)
                .bind()
        customListsRepository
            .deleteCustomList(action.id)
            .mapLeft(DeleteWithUndoError::Delete)
            .bind()
        Deleted(undo = action.not(locations = customList.locations, name = customList.name))
    }

    suspend fun performAction(
        action: CustomListAction.UpdateLocations
    ): Either<UpdateLocationsError, LocationsChanged> = either {
        val customList =
            customListsRepository
                .getCustomListById(action.id)
                .mapLeft(UpdateLocationsError::Fetch)
                .bind()
        customListsRepository
            .updateCustomListLocations(action.id, action.locations)
            .mapLeft(UpdateLocationsError::UpdateError)
            .bind()
        LocationsChanged(
            name = customList.name,
            undo = action.not(locations = customList.locations)
        )
    }
}

sealed interface CustomListActionError

sealed interface CreateWithLocationsError : CustomListActionError {

    data class CreateActionError(val error: CreateCustomListError) : CreateWithLocationsError

    data class UpdateLocationsActionError(val error: UpdateCustomListLocationsError) :
        CreateWithLocationsError

    data object UnableToFetchRelayList : CreateWithLocationsError
}

sealed interface DeleteWithUndoError : CustomListActionError {
    data class Fetch(val getCustomListError: GetCustomListError) : DeleteWithUndoError

    data class Delete(val deleteCustomListError: DeleteCustomListError) : DeleteWithUndoError
}

data class RenameError(val error: UpdateCustomListNameError) : CustomListActionError

sealed interface UpdateLocationsError : CustomListActionError {

    data class Fetch(val getCustomListError: GetCustomListError) : UpdateLocationsError

    data class UpdateError(val updateCustomListLocationsError: UpdateCustomListLocationsError) :
        UpdateLocationsError
}
