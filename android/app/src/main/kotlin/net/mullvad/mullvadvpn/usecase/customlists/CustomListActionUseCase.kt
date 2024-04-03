package net.mullvad.mullvadvpn.usecase.customlists

import arrow.core.Either
import arrow.core.raise.either
import kotlinx.coroutines.flow.firstOrNull
import net.mullvad.mullvadvpn.compose.communication.CustomListAction
import net.mullvad.mullvadvpn.compose.communication.CustomListResult
import net.mullvad.mullvadvpn.model.CreateCustomListError
import net.mullvad.mullvadvpn.model.DeleteCustomListError
import net.mullvad.mullvadvpn.model.GetCustomListError
import net.mullvad.mullvadvpn.model.ModifyCustomListError
import net.mullvad.mullvadvpn.relaylist.getRelayItemsByCodes
import net.mullvad.mullvadvpn.repository.CustomListsRepository
import net.mullvad.mullvadvpn.usecase.RelayListUseCase

class CustomListActionUseCase(
    private val customListsRepository: CustomListsRepository,
    private val relayListUseCase: RelayListUseCase
) {
    suspend fun performAction(action: CustomListAction): Either<Any, CustomListResult> {
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
    ): Either<ModifyCustomListError, CustomListResult.Renamed> =
        customListsRepository.updateCustomListName(action.id, action.newName).map {
            CustomListResult.Renamed(undo = action.not())
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
                    .mapLeft(CreateCustomListWithLocationsError::Update)
                    .bind()

                relayListUseCase
                    .relayList()
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
    ): Either<ModifyCustomListError, CustomListResult.LocationsChanged> = either {
        val customList = customListsRepository.getCustomListById(action.id).bind()
        customListsRepository.updateCustomListLocations(action.id, action.locations).bind()
        CustomListResult.LocationsChanged(
            name = customList.name,
            undo = action.not(locations = customList.locations)
        )
    }
}

sealed interface CreateCustomListWithLocationsError {
    data class Create(val error: CreateCustomListError) : CreateCustomListWithLocationsError

    data class Update(val error: ModifyCustomListError) : CreateCustomListWithLocationsError

    data object UnableToFetchRelayList : CreateCustomListWithLocationsError
}

sealed interface DeleteCustomListWithUndoError {
    data class Delete(val error: DeleteCustomListError) : DeleteCustomListWithUndoError

    data class Get(val error: GetCustomListError) : DeleteCustomListWithUndoError
}
