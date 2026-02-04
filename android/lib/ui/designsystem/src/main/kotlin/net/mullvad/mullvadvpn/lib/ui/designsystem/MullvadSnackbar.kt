package net.mullvad.mullvadvpn.lib.ui.designsystem

import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Snackbar
import androidx.compose.material3.SnackbarData
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier

@Composable
fun MullvadSnackbar(modifier: Modifier = Modifier, snackbarData: SnackbarData) {
    Snackbar(
        modifier = modifier,
        snackbarData = snackbarData,
        containerColor = MaterialTheme.colorScheme.surfaceContainer,
        contentColor = MaterialTheme.colorScheme.onSurface,
        actionColor = MaterialTheme.colorScheme.onSurface,
        dismissActionContentColor = MaterialTheme.colorScheme.onSurface,
    )
}
