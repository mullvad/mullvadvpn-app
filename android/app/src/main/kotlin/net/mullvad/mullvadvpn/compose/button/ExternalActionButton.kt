package net.mullvad.mullvadvpn.compose.button

import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.lib.theme.AppTheme

@Preview
@Composable
private fun PreviewExternalButton() {
    AppTheme { ExternalButton(onClick = {}, text = "Button") }
}

@Composable
fun ExternalButton(
    onClick: () -> Unit,
    text: String,
    modifier: Modifier = Modifier,
    isEnabled: Boolean = true,
) {
    VariantButton(
        text = text,
        onClick = onClick,
        modifier = modifier,
        isEnabled = isEnabled,
        icon = R.drawable.icon_extlink,
    )
}
