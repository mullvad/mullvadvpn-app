package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.usecase.customlists.RenameError

data class EditCustomListNameUiState(val name: String = "", val error: RenameError? = null) {
    val isValidName = name.isNotBlank()
}
