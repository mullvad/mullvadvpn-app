package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.usecase.customlists.RenameCustomListError

data class EditCustomListNameUiState(
    val name: String = "",
    val error: RenameCustomListError? = null
)
