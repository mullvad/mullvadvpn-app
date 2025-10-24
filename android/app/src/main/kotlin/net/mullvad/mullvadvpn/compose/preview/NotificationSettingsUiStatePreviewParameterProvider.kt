package net.mullvad.mullvadvpn.compose.preview

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.NotificationSettingsUiState
import net.mullvad.mullvadvpn.util.Lc

class NotificationSettingsUiStatePreviewParameterProvider :
    PreviewParameterProvider<Lc<Unit, NotificationSettingsUiState>> {
    override val values: Sequence<Lc<Unit, NotificationSettingsUiState>> =
        sequenceOf(
            Lc.Loading(Unit),
            Lc.Content(NotificationSettingsUiState(locationInNotificationEnabled = true)),
        )
}
