package net.mullvad.mullvadvpn.usecase.customlists

import arrow.core.Either
import arrow.core.raise.either
import kotlinx.coroutines.flow.firstOrNull
import net.mullvad.mullvadvpn.compose.communication.CustomListAction
import net.mullvad.mullvadvpn.compose.communication.CustomListResult
import net.mullvad.mullvadvpn.model.CreateCustomListError
import net.mullvad.mullvadvpn.model.CustomListId
import net.mullvad.mullvadvpn.model.DeleteCustomListError
import net.mullvad.mullvadvpn.model.GetCustomListError
import net.mullvad.mullvadvpn.model.ModifyCustomListError
import net.mullvad.mullvadvpn.model.UpdateCustomListError
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
    ): Either<RenameCustomListError, CustomListResult.Renamed> =
        customListsRepository
            .updateCustomListName(action.id, action.newName)
            .map { CustomListResult.Renamed(undo = action.not()) }
            .mapLeft {
                when (it) {
                    is GetCustomListError -> RenameCustomListError.NotFound(action.id)
                    is UpdateCustomListError.NameAlreadyExists ->
                        RenameCustomListError.NameAlreadyExists(action.newName.value)
                    is UpdateCustomListError.Unknown ->
                        RenameCustomListError.Unknown(it.throwable.message ?: "", it.throwable)
                }
            }

    suspend fun performAction(
        action: CustomListAction.Create
    ): Either<CreateCustomListWithLocationsError, CustomListResult.Created> = either {
        val customListId =
            customListsRepository
                .createCustomList(action.name)
                .mapLeft(CreateCustomListWithLocationsError::Create)
                .bind()

        val locationNames =
            if (action.locations.isNotEmpty()) {
                customListsRepository
                    .updateCustomListLocations(customListId, action.locations)
                    .mapLeft(CreateCustomListWithLocationsError::Modify)
                    .bind()

                relayListRepository.relayList
                    .firstOrNull()
                    ?.getRelayItemsByCodes(action.locations)
                    ?.map { it.name }
                    ?: raise(CreateCustomListWithLocationsError.UnableToFetchRelayList)
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
    ): Either<DeleteCustomListWithUndoError, CustomListResult.Deleted> = either {
        val customList =
            customListsRepository
                .getCustomListById(action.id)
                .mapLeft(DeleteCustomListWithUndoError::Get)
                .bind()
        customListsRepository
            .deleteCustomList(action.id)
            .mapLeft(DeleteCustomListWithUndoError::Delete)
            .bind()
        CustomListResult.Deleted(
            undo = action.not(locations = customList.locations, name = customList.name)
        )
    }

    suspend fun performAction(
        action: CustomListAction.UpdateLocations
    ): Either<UpdateLocationsCustomListError, CustomListResult.LocationsChanged> = either {
        val customList =
            customListsRepository
                .getCustomListById(action.id)
                .mapLeft(UpdateLocationsCustomListError::Get)
                .bind()
        customListsRepository
            .updateCustomListLocations(action.id, action.locations)
            .mapLeft(UpdateLocationsCustomListError::Modify)
            .bind()
        CustomListResult.LocationsChanged(
            name = customList.name,
            undo = action.not(locations = customList.locations)
        )
    }
}

sealed interface CustomListActionError

sealed interface CreateCustomListWithLocationsError : CustomListActionError {
    data class Create(val error: CreateCustomListError) : CreateCustomListWithLocationsError

    data class Modify(val error: ModifyCustomListError) : CreateCustomListWithLocationsError

    data object UnableToFetchRelayList : CreateCustomListWithLocationsError
}

sealed interface DeleteCustomListWithUndoError : CustomListActionError {
    data class Delete(val error: DeleteCustomListError) : DeleteCustomListWithUndoError

    data class Get(val error: GetCustomListError) : DeleteCustomListWithUndoError
}

sealed interface RenameCustomListError : CustomListActionError {
    data class NotFound(val id: CustomListId) : RenameCustomListError

    data class NameAlreadyExists(val name: String) : RenameCustomListError

    data class Unknown(val message: String, val throwable: Throwable? = null) :
        RenameCustomListError
}

sealed interface UpdateLocationsCustomListError : CustomListActionError {
    data class Modify(val error: ModifyCustomListError) : UpdateLocationsCustomListError

    data class Get(val error: GetCustomListError) : UpdateLocationsCustomListError
}
