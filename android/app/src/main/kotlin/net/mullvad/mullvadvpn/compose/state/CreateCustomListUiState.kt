package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.usecase.customlists.CustomListActionError

data class CreateCustomListUiState(val error: CustomListActionError.CreateWithLocations? = null)
