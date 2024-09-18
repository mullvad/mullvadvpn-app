package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.CustomListName
import net.mullvad.mullvadvpn.lib.model.GeoLocationId

sealed interface EditCustomListUiState {
    data object Loading : EditCustomListUiState

    data object NotFound : EditCustomListUiState

    data class Content(
        val id: CustomListId,
        val name: CustomListName,
        val locations: List<GeoLocationId>,
    ) : EditCustomListUiState
}
