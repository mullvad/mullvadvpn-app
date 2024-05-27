package net.mullvad.mullvadvpn.compose.util

import androidx.compose.material3.SnackbarDuration
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.SnackbarResult
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.launch

@Suppress("LongParameterList")
suspend fun SnackbarHostState.showSnackbarImmediately(
    message: String,
    actionLabel: String? = null,
    onAction: (() -> Unit) = {},
    withDismissAction: Boolean = false,
    onDismiss: (() -> Unit) = {},
    duration: SnackbarDuration =
        if (actionLabel == null) SnackbarDuration.Short else SnackbarDuration.Indefinite
) = coroutineScope {
    launch {
        currentSnackbarData?.dismiss()
        when (showSnackbar(message, actionLabel, withDismissAction, duration)) {
            SnackbarResult.ActionPerformed -> onAction()
            SnackbarResult.Dismissed -> onDismiss()
        }
    }
}
