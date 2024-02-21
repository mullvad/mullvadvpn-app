package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.model.CustomListsError

data class UpdateCustomListUiState(val error: CustomListsError? = null)
