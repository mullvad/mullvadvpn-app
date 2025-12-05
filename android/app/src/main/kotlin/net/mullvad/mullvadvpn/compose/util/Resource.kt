package net.mullvad.mullvadvpn.compose.util

import androidx.annotation.StringRes
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.platform.LocalResources
import androidx.compose.ui.text.AnnotatedString
import net.mullvad.mullvadvpn.util.toAnnotatedString

@Composable
fun annotatedStringResource(@StringRes id: Int): AnnotatedString {
    val resources = LocalResources.current
    return remember(id) { resources.getText(id).toAnnotatedString() }
}
