package net.mullvad.mullvadvpn.repository

import arrow.core.Either
import arrow.core.raise.either
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.mapNotNull
import kotlinx.coroutines.flow.stateIn
import net.mullvad.mullvadvpn.lib.common.util.firstOrNullWithTimeout
import net.mullvad.mullvadvpn.lib.daemon.grpc.ManagementService
import net.mullvad.mullvadvpn.lib.model.CustomList
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.CustomListName
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.GetCustomListError
import net.mullvad.mullvadvpn.lib.model.ModifyCustomListError

class CustomListsRepository(
    private val managementService: ManagementService,
    dispatcher: CoroutineDispatcher = Dispatchers.IO,
) {
    val customLists: StateFlow<List<CustomList>?> =
        managementService.settings
            .mapNotNull { it.customLists }
            .stateIn(CoroutineScope(dispatcher), SharingStarted.Eagerly, null)

    suspend fun createCustomList(name: CustomListName) = managementService.createCustomList(name)

    suspend fun deleteCustomList(id: net.mullvad.mullvadvpn.lib.model.CustomListId) =
        managementService.deleteCustomList(id)

    private suspend fun updateCustomList(customList: CustomList) =
        managementService.updateCustomList(customList)

    suspend fun updateCustomListName(
        id: net.mullvad.mullvadvpn.lib.model.CustomListId,
        name: CustomListName
    ): Either<ModifyCustomListError, Unit> = either {
        val customList = getCustomListById(id).bind()
        updateCustomList(customList.copy(name = name)).bind()
    }

    suspend fun updateCustomListLocations(
        id: net.mullvad.mullvadvpn.lib.model.CustomListId,
        locations: List<net.mullvad.mullvadvpn.lib.model.GeoLocationId>
    ): Either<ModifyCustomListError, Unit> = either {
        val customList = getCustomListById(id).bind()
        updateCustomList(customList.copy(locations = locations)).bind()
    }

    suspend fun getCustomListById(
        id: net.mullvad.mullvadvpn.lib.model.CustomListId
    ): Either<GetCustomListError, CustomList> =
        either {
                customLists
                    .mapNotNull { it?.find { customList -> customList.id == id } }
                    .firstOrNullWithTimeout(GET_CUSTOM_LIST_TIMEOUT_MS)
                    ?: raise(GetCustomListError(id))
            }
            .mapLeft { GetCustomListError(id) }

    companion object {
        private const val GET_CUSTOM_LIST_TIMEOUT_MS = 5000L
    }
}
