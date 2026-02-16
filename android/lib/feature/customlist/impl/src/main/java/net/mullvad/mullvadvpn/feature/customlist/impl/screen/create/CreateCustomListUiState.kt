package net.mullvad.mullvadvpn.feature.customlist.impl.screen.create

import net.mullvad.mullvadvpn.lib.usecase.customlists.CreateWithLocationsError

data class CreateCustomListUiState(val error: CreateWithLocationsError? = null)
