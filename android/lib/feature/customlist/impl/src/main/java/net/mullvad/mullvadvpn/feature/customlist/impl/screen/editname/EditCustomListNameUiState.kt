package net.mullvad.mullvadvpn.feature.customlist.impl.screen.editname

import net.mullvad.mullvadvpn.lib.usecase.customlists.RenameError

data class EditCustomListNameUiState(val name: String = "", val error: RenameError? = null) {
    val isValidName = name.isNotBlank()
}
