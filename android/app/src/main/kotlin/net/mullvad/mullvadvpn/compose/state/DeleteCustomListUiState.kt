package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.lib.model.CustomListName
import net.mullvad.mullvadvpn.usecase.customlists.DeleteWithUndoError

data class DeleteCustomListUiState(val name: CustomListName, val deleteError: DeleteWithUndoError?)
