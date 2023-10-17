package net.mullvad.mullvadvpn.compose

import androidx.compose.runtime.Composable
import androidx.compose.ui.test.junit4.ComposeContentTestRule
import net.mullvad.mullvadvpn.lib.theme.AppTheme

fun ComposeContentTestRule.setContentWithTheme(content: @Composable () -> Unit) {
    setContent { AppTheme { content() } }
}
