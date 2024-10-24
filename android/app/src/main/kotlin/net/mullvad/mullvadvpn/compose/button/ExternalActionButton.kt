package net.mullvad.mullvadvpn.compose.button

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.OpenInNew
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.tooling.preview.Preview
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
            isEnabled = true,
        )
    }
}

@Preview
@Composable
private fun PreviewExternalButtonSpinner() {
    AppTheme {
        ExternalButton(
            onClick = {},
            text = "Button text is long and is trying to take up space that is large",
            isEnabled = true,
            isLoading = true,
        )
    }
}

@Composable
fun ExternalButton(
    onClick: () -> Unit,
    text: String,
    modifier: Modifier = Modifier,
    isEnabled: Boolean = true,
    isLoading: Boolean = false,
) {
    VariantButton(
        text = text,
        onClick = onClick,
        modifier = modifier,
        isEnabled = isEnabled,
        isLoading = isLoading,
        icon = {
            if (!isLoading) {
                Icon(
                    imageVector = Icons.AutoMirrored.Filled.OpenInNew,
                    tint = MaterialTheme.colorScheme.onTertiary,
                    contentDescription = null,
                )
            }
        },
    )
}
