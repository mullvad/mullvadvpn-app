package net.mullvad.mullvadvpn.feature.customlist.impl.screen.delete

import net.mullvad.mullvadvpn.lib.model.CustomListName
import net.mullvad.mullvadvpn.lib.usecase.customlists.DeleteWithUndoError

data class DeleteCustomListUiState(val name: CustomListName, val deleteError: DeleteWithUndoError?)
