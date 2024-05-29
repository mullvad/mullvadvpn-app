package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.usecase.customlists.CustomListActionError

data class EditCustomListNameUiState(
    val name: String = "",
    val error: CustomListActionError.Rename? = null
)
