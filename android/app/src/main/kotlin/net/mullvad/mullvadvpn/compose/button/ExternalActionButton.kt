package net.mullvad.mullvadvpn.compose.button

import androidx.compose.material3.Icon
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.lib.theme.AppTheme

@Preview
@Composable
private fun PreviewExternalButtonEnabled() {
    AppTheme { ExternalButton(onClick = {}, text = "Button", isEnabled = true) }
}

@Preview
@Composable
private fun PreviewExternalButtonDisabled() {
    AppTheme { ExternalButton(onClick = {}, text = "Button", isEnabled = false) }
}

@Preview
@Composable
private fun PreviewExternalButtonLongText() {
    AppTheme {
        ExternalButton(
            onClick = {},
            text = "Button text is long and is trying to take up space that is large",
            isEnabled = true
        )
    }
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
        icon = {
            Icon(painter = painterResource(id = R.drawable.icon_extlink), contentDescription = null)
        },
    )
}
