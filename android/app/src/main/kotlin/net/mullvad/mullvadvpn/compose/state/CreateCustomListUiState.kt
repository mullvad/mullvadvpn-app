package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.lib.usecase.customlists.CreateWithLocationsError

data class CreateCustomListUiState(val error: CreateWithLocationsError? = null)
