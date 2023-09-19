package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.foundation.layout.size
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.window.DialogProperties
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens

@Preview
@Composable
private fun PreviewLoadingDialog() {
    AppTheme { LoadingDialog(text = "Loading") }
}

@Composable
fun LoadingDialog(
    text: String,
    dismissOnClickOutside: Boolean = false,
    dismissOnBackPress: Boolean = false,
) {
    AlertDialog(
        onDismissRequest = {},
        icon = {
            CircularProgressIndicator(
                color = MaterialTheme.colorScheme.onBackground,
                modifier = Modifier.size(Dimens.progressIndicatorSize)
            )
        },
        title = { Text(text = text) },
        confirmButton = {},
        properties =
            DialogProperties(
                dismissOnClickOutside = dismissOnClickOutside,
                dismissOnBackPress = dismissOnBackPress,
            ),
        containerColor = MaterialTheme.colorScheme.background
    )
}
