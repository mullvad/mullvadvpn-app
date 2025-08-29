package net.mullvad.mullvadvpn.compose.preview

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.applist.AppData
import net.mullvad.mullvadvpn.util.Lc
import net.mullvad.mullvadvpn.util.toLc
import net.mullvad.mullvadvpn.viewmodel.Loading
import net.mullvad.mullvadvpn.viewmodel.SplitTunnelingUiState

class SplitTunnelingUiStatePreviewParameterProvider :
    PreviewParameterProvider<Lc<Loading, SplitTunnelingUiState>> {
    override val values =
        sequenceOf(
            SplitTunnelingUiState(
                    enabled = true,
                    excludedApps =
                        listOf(
                            AppData(
                                packageName = "my.package.a",
                                name = "TitleA",
                                iconRes = R.drawable.ic_icons_missing,
                            ),
                            AppData(
                                packageName = "my.package.b",
                                name = "TitleB",
                                iconRes = R.drawable.ic_icons_missing,
                            ),
                        ),
                    includedApps =
                        listOf(
                            AppData(
                                packageName = "my.package.c",
                                name = "TitleC",
                                iconRes = R.drawable.ic_icons_missing,
                            )
                        ),
                    showSystemApps = true,
                )
                .toLc(),
            Lc.Loading(Loading(enabled = true)),
        )
}
