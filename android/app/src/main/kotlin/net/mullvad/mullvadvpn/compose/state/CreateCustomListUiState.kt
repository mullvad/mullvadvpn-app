package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.model.CustomListsError

data class CreateCustomListUiState(val error: CustomListsError? = null)
