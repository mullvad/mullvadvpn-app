package net.mullvad.mullvadvpn.compose.extensions

import androidx.compose.material3.SnackbarDuration
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.SnackbarResult

suspend fun SnackbarHostState.showSnackbar(
    message: String,
    actionLabel: String,
    duration: SnackbarDuration = SnackbarDuration.Indefinite,
    onAction: (() -> Unit),
    onDismiss: (() -> Unit) = {}
) {
    when (showSnackbar(message = message, actionLabel = actionLabel, duration = duration)) {
        SnackbarResult.ActionPerformed -> onAction()
        SnackbarResult.Dismissed -> onDismiss()
    }
}
