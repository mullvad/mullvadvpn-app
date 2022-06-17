package net.mullvad.mullvadvpn.compose.component

import android.text.Spanned
import androidx.annotation.StringRes
import androidx.compose.runtime.Composable
import androidx.compose.runtime.ReadOnlyComposable
import androidx.compose.ui.platform.LocalContext
import androidx.core.text.HtmlCompat

@Composable
@ReadOnlyComposable
fun textResource(@StringRes id: Int, vararg formatArgs: Any): Spanned {
    return LocalContext.current.resources.getString(id, *formatArgs).let { text ->
        HtmlCompat.fromHtml(text, HtmlCompat.FROM_HTML_MODE_COMPACT)
    }
}
