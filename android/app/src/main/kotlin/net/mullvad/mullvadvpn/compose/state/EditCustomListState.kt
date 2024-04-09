package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.model.CustomListId
import net.mullvad.mullvadvpn.model.GeographicLocationConstraint

sealed interface EditCustomListState {
    data object Loading : EditCustomListState

    data object NotFound : EditCustomListState

    data class Content(
        val id: CustomListId,
        val name: String,
        val locations: List<GeographicLocationConstraint>
    ) : EditCustomListState
}
