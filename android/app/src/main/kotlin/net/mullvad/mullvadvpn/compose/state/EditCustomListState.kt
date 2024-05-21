package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.CustomListName
import net.mullvad.mullvadvpn.lib.model.GeoLocationId

sealed interface EditCustomListState {
    data object Loading : EditCustomListState

    data object NotFound : EditCustomListState

    data class Content(
        val id: net.mullvad.mullvadvpn.lib.model.CustomListId,
        val name: CustomListName,
        val locations: List<net.mullvad.mullvadvpn.lib.model.GeoLocationId>
    ) : EditCustomListState
}
