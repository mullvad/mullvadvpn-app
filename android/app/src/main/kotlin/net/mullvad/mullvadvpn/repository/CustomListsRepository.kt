package net.mullvad.mullvadvpn.repository

import arrow.core.Either
import arrow.core.raise.either
import co.touchlab.kermit.Logger
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.mapNotNull
import kotlinx.coroutines.flow.stateIn
import net.mullvad.mullvadvpn.lib.daemon.grpc.ManagementService
import net.mullvad.mullvadvpn.lib.model.CustomList
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.CustomListName
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.GetCustomListError
import net.mullvad.mullvadvpn.lib.model.UpdateCustomListLocationsError
import net.mullvad.mullvadvpn.lib.model.UpdateCustomListNameError

class CustomListsRepository(
    private val managementService: ManagementService,
    dispatcher: CoroutineDispatcher = Dispatchers.IO,
) {
    val customLists: StateFlow<List<CustomList>?> =
        managementService.settings
            .mapNotNull { it.customLists }
            .stateIn(CoroutineScope(dispatcher), SharingStarted.Eagerly, null)

    suspend fun createCustomList(name: CustomListName) = managementService.createCustomList(name)

    suspend fun deleteCustomList(id: CustomListId) = managementService.deleteCustomList(id)

    suspend fun updateCustomList(customList: CustomList) =
        managementService.updateCustomList(customList)

    suspend fun updateCustomListName(
        id: CustomListId,
        name: CustomListName,
    ): Either<UpdateCustomListNameError, Unit> = either {
        val customList = getCustomListById(id).bind()
        updateCustomList(customList.copy(name = name))
            .mapLeft(UpdateCustomListNameError::from)
            .bind()
    }

    suspend fun updateCustomListLocations(
        id: CustomListId,
        locations: List<GeoLocationId>,
    ): Either<UpdateCustomListLocationsError, Unit> = either {
        val customList = getCustomListById(id).bind()
        updateCustomList(customList.copy(locations = locations))
            .mapLeft(UpdateCustomListLocationsError::from)
            .bind()
    }

    /**
     * There is no guarantee this will return a up to date custom list. E.g if you invoked
     * updateCustomList just before this you might get an out of date value.
     */
    fun getCustomListById(id: CustomListId): Either<GetCustomListError, CustomList> = either {
        val customLists =
            customLists.value
                ?: raise(GetCustomListError(id)).also { Logger.e("Custom lists never loaded") }
        customLists.firstOrNull { customList -> customList.id == id }
            ?: raise(GetCustomListError(id))
    }
}
