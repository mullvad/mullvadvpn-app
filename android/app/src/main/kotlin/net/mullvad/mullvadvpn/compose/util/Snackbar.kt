package net.mullvad.mullvadvpn.compose.util

import androidx.compose.material3.SnackbarDuration
import androidx.compose.material3.SnackbarHostState
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch

suspend fun SnackbarHostState.showSnackbarImmediately(
    coroutineScope: CoroutineScope,
    message: String,
    actionLabel: String? = null,
    withDismissAction: Boolean = false,
    duration: SnackbarDuration =
        if (actionLabel == null) SnackbarDuration.Short else SnackbarDuration.Indefinite
) =
    coroutineScope.launch {
        currentSnackbarData?.dismiss()
        showSnackbar(message, actionLabel, withDismissAction, duration)
    }
