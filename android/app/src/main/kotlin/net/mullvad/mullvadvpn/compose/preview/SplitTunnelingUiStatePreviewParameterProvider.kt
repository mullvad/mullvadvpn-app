package net.mullvad.mullvadvpn.compose.preview

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.applist.AppData
import net.mullvad.mullvadvpn.compose.state.SplitTunnelingUiState

class SplitTunnelingUiStatePreviewParameterProvider :
    PreviewParameterProvider<SplitTunnelingUiState> {
    override val values =
        sequenceOf(
            SplitTunnelingUiState.ShowAppList(
                enabled = true,
                excludedApps =
                    listOf(
                        AppData(
                            packageName = "my.package.a",
                            name = "TitleA",
                            iconRes = R.drawable.icon_alert,
                        ),
                        AppData(
                            packageName = "my.package.b",
                            name = "TitleB",
                            iconRes = R.drawable.icon_chevron,
                        ),
                    ),
                includedApps =
                    listOf(
                        AppData(
                            packageName = "my.package.c",
                            name = "TitleC",
                            iconRes = R.drawable.icon_alert,
                        )
                    ),
                showSystemApps = true,
            ),
            SplitTunnelingUiState.Loading(enabled = true),
        )
}
