package net.mullvad.mullvadvpn.compose.component

import androidx.annotation.StringRes
import androidx.compose.runtime.Composable
import androidx.compose.runtime.ReadOnlyComposable
import androidx.compose.ui.platform.LocalResources

@Composable
@ReadOnlyComposable
fun textResource(@StringRes id: Int, vararg formatArgs: Any): String {
    return LocalResources.current.getString(id, *formatArgs)
}
