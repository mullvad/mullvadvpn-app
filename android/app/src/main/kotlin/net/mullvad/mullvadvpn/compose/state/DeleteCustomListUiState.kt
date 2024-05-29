package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.usecase.customlists.CustomListActionError.DeleteWithUndo

data class DeleteCustomListUiState(val deleteError: DeleteWithUndo?)
