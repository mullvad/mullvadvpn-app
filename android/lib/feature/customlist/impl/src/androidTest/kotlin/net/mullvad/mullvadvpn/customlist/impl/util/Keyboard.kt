package net.mullvad.mullvadvpn.customlist.impl.util

import androidx.compose.runtime.Composable
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.platform.InterceptPlatformTextInput
import kotlinx.coroutines.awaitCancellation

// Stackoverflow answer: https://stackoverflow.com/a/78720287
@OptIn(ExperimentalComposeUiApi::class)
@Composable
fun DisableSoftKeyboard(disable: Boolean = true, content: @Composable () -> Unit) {
    InterceptPlatformTextInput(
        interceptor = { request, nextHandler ->
            if (!disable) {
                nextHandler.startInputMethod(request)
            } else {
                awaitCancellation()
            }
        },
        content = content,
    )
}
