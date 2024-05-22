package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.usecase.customlists.DeleteCustomListWithUndoError

data class DeleteCustomListUiState(val deleteError: DeleteCustomListWithUndoError?)
