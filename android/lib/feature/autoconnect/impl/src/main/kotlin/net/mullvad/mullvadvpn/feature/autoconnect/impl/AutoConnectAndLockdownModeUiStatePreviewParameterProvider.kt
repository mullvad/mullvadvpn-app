package net.mullvad.mullvadvpn.feature.autoconnect.impl

import androidx.compose.ui.tooling.preview.PreviewParameterProvider

class AutoConnectAndLockdownModeUiStatePreviewParameterProvider :
    PreviewParameterProvider<AutoConnectAndLockdownModeUiState> {
    override val values: Sequence<AutoConnectAndLockdownModeUiState> =
        sequenceOf(
            AutoConnectAndLockdownModeUiState(false),
            AutoConnectAndLockdownModeUiState(true),
        )
}
