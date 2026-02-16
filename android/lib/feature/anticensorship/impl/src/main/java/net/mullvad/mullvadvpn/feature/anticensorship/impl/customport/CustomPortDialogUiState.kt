package net.mullvad.mullvadvpn.feature.anticensorship.impl.customport

import net.mullvad.mullvadvpn.lib.model.PortRange

data class CustomPortDialogUiState(
    val portInput: String,
    val isValidInput: Boolean,
    val allowedPortRanges: List<PortRange>,
    val recommendedPortRanges: List<PortRange>,
    val showResetToDefault: Boolean,
)
