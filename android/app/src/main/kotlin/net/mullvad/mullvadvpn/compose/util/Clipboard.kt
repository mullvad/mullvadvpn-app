package net.mullvad.mullvadvpn.compose.util

import android.content.ClipData
import android.os.Build
import android.os.PersistableBundle
import androidx.compose.material3.SnackbarDuration
import androidx.compose.material3.SnackbarHostState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.platform.Clipboard
import androidx.compose.ui.platform.LocalClipboard
import androidx.compose.ui.platform.toClipEntry
import kotlinx.coroutines.launch

typealias CopyToClipboardHandle = (content: String, toastMessage: String?) -> Unit

private const val IS_SENSITIVE_FLAG = "android.content.extra.IS_SENSITIVE"

@Composable
fun createCopyToClipboardHandle(
    snackbarHostState: SnackbarHostState,
    isSensitive: Boolean,
): CopyToClipboardHandle {
    val scope = rememberCoroutineScope()
    val clipboard: Clipboard = LocalClipboard.current

    return { textToCopy: String, toastMessage: String? ->
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.TIRAMISU && toastMessage != null) {
            scope.launch {
                snackbarHostState.showSnackbarImmediately(
                    message = toastMessage,
                    duration = SnackbarDuration.Short,
                )
            }
        }

        scope.launch {
            val clip =
                ClipData.newPlainText("", textToCopy)
                    .apply {
                        description.extras =
                            PersistableBundle().apply { putBoolean(IS_SENSITIVE_FLAG, isSensitive) }
                    }
                    .toClipEntry()

            clipboard.setClipEntry(clipEntry = clip)
        }
    }
}
