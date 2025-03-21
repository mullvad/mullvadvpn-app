package net.mullvad.mullvadvpn.compose.util

import android.content.ClipData
import android.os.Build
import android.os.PersistableBundle
import androidx.compose.material3.SnackbarDuration
import androidx.compose.material3.SnackbarHostState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.platform.ClipboardManager
import androidx.compose.ui.platform.LocalClipboardManager
import androidx.compose.ui.platform.toClipEntry
import kotlinx.coroutines.launch

typealias CopyToClipboardHandle = (content: String, toastMessage: String?) -> Unit

@Composable
fun createCopyToClipboardHandle(
    snackbarHostState: SnackbarHostState,
    isSensitive: Boolean,
): CopyToClipboardHandle {
    val scope = rememberCoroutineScope()
    val clipboardManager: ClipboardManager = LocalClipboardManager.current

    return { textToCopy: String, toastMessage: String? ->
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.TIRAMISU && toastMessage != null) {
            scope.launch {
                snackbarHostState.showSnackbarImmediately(
                    message = toastMessage,
                    duration = SnackbarDuration.Short,
                )
            }
        }

        val clip =
            ClipData.newPlainText("", textToCopy)
                .apply {
                    description.extras =
                        PersistableBundle().apply {
                            putBoolean("android.content.extra.IS_SENSITIVE", isSensitive)
                        }
                }
                .toClipEntry()

        clipboardManager.setClip(clip)
    }
}
