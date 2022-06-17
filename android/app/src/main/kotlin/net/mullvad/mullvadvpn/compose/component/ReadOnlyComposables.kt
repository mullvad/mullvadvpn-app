package net.mullvad.mullvadvpn.compose.component

import androidx.annotation.StringRes
import androidx.compose.runtime.Composable
import androidx.compose.runtime.ReadOnlyComposable
import androidx.compose.ui.platform.LocalContext

@Composable
@ReadOnlyComposable
fun textResource(@StringRes id: Int, vararg formatArgs: Any): String {
    return LocalContext.current.resources.getString(id, *formatArgs)
}
