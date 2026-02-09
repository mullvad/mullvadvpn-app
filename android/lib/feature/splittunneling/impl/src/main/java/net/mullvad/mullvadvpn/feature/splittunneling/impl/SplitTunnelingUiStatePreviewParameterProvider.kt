package net.mullvad.mullvadvpn.feature.splittunneling.impl

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.feature.splittunneling.impl.applist.AppData
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.common.toLc
import net.mullvad.mullvadvpn.lib.ui.resource.R

class SplitTunnelingUiStatePreviewParameterProvider :
    PreviewParameterProvider<Lc<Loading, SplitTunnelingUiState>> {
    override val values =
        sequenceOf(
            SplitTunnelingUiState(
                    enabled = true,
                    excludedApps = excludedApps,
                    includedApps = includedApps,
                    showSystemApps = true,
                )
                .toLc(),
            SplitTunnelingUiState(
                    enabled = true,
                    excludedApps = excludedApps,
                    includedApps = includedApps.filter { !it.isSystemApp },
                    showSystemApps = false,
                )
                .toLc(),
            Lc.Loading(Loading(enabled = true)),
        )
}

private val excludedApps =
    listOf(
        AppData(packageName = "my.package.a", name = "TitleA", iconRes = R.drawable.icon_android),
        AppData(packageName = "my.package.b", name = "TitleB", iconRes = R.drawable.icon_android),
        AppData(
            packageName = "my.package.c",
            name = "TitleC (System app)",
            iconRes = R.drawable.icon_android,
            isSystemApp = true,
        ),
    )
private val includedApps =
    listOf(
        AppData(packageName = "my.package.d", name = "TitleD", iconRes = R.drawable.icon_android),
        AppData(
            packageName = "my.package.e",
            name = "TitleE (System app)",
            iconRes = R.drawable.icon_android,
            isSystemApp = true,
        ),
    )
