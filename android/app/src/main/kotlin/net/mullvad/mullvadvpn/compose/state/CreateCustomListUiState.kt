package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.usecase.customlists.CreateWithLocationsError

data class CreateCustomListUiState(val error: CreateWithLocationsError? = null)
