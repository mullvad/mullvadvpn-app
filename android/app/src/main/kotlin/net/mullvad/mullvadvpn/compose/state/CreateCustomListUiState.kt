package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.model.CreateCustomListError
import net.mullvad.mullvadvpn.usecase.customlists.CreateCustomListWithLocationsError

data class CreateCustomListUiState(val error: CreateCustomListWithLocationsError? = null)
