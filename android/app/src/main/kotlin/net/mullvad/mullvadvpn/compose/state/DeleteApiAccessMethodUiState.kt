package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.lib.model.RemoveApiAccessMethodError

data class DeleteApiAccessMethodUiState(val deleteError: RemoveApiAccessMethodError?)
