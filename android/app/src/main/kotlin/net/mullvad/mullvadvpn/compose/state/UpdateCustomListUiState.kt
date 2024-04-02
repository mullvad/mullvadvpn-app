package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.model.ModifyCustomListError

data class UpdateCustomListUiState(val name: String = "", val error: ModifyCustomListError? = null)
