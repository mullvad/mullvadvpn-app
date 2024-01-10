package net.mullvad.mullvadvpn.compose

import androidx.compose.runtime.Composable
import de.mannodermaus.junit5.compose.ComposeContext
import net.mullvad.mullvadvpn.lib.theme.AppTheme

fun ComposeContext.setContentWithTheme(content: @Composable () -> Unit) {
    setContent { AppTheme { content() } }
}
