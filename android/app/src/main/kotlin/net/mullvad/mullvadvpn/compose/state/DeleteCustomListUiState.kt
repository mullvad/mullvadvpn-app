package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.usecase.customlists.DeleteWithUndoError

data class DeleteCustomListUiState(val deleteError: DeleteWithUndoError?)
