package net.mullvad.mullvadvpn.compose.util

import android.os.Build
import androidx.compose.material3.SnackbarDuration
import androidx.compose.material3.SnackbarHostState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.platform.ClipboardManager
import androidx.compose.ui.platform.LocalClipboardManager
import androidx.compose.ui.text.AnnotatedString

typealias CopyToClipboardHandle = (content: String, toastMessage: String?) -> Unit

@Composable
fun createCopyToClipboardHandle(
    snackbarHostState: SnackbarHostState,
): CopyToClipboardHandle {
    val scope = rememberCoroutineScope()
    val clipboardManager: ClipboardManager = LocalClipboardManager.current

    return { textToCopy: String, toastMessage: String? ->
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.TIRAMISU && toastMessage != null) {
            snackbarHostState.showSnackbarImmediately(
                coroutineScope = scope,
                message = toastMessage,
                duration = SnackbarDuration.Short
            )
        }

        clipboardManager.setText(AnnotatedString(textToCopy))
    }
}
