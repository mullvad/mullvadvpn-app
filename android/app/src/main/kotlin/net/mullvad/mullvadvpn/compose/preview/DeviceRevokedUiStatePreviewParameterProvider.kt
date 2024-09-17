package net.mullvad.mullvadvpn.compose.preview

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.DeviceRevokedUiState

class DeviceRevokedUiStatePreviewParameterProvider :
    PreviewParameterProvider<DeviceRevokedUiState> {
    override val values =
        sequenceOf(
            DeviceRevokedUiState.SECURED,
            DeviceRevokedUiState.UNSECURED,
            DeviceRevokedUiState.UNKNOWN,
        )
}
