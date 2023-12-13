package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.runtime.Composable
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.component.AutoConnectCarousel
import net.mullvad.mullvadvpn.compose.component.NavigateBackIconButton
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithLargeTopBarAndButton
import net.mullvad.mullvadvpn.lib.common.util.openVpnSettings
import net.mullvad.mullvadvpn.lib.theme.AppTheme

@Preview
@Composable
private fun PreviewAutoConnectAndLockdownModeScreen() {

    AppTheme {
        AutoConnectAndLockdownModeScreen(
            onBackClick = {},
        )
    }
}

@Composable
fun AutoConnectAndLockdownModeScreen(
    onBackClick: () -> Unit = {},
) {
    val context = LocalContext.current
    ScaffoldWithLargeTopBarAndButton(
        appBarTitle = stringResource(id = R.string.auto_connect_and_lockdown_mode_two_lines),
        navigationIcon = { NavigateBackIconButton(onBackClick) },
        buttonTitle = stringResource(id = R.string.go_to_vpn_settings),
        onButtonClick = { context.openVpnSettings() },
        content = { modifier -> AutoConnectCarousel() }
    )
}
