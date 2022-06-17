package net.mullvad.mullvadvpn.compose.component

import android.text.Spanned
import android.util.TypedValue
import android.widget.TextView
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.viewinterop.AndroidView

@Composable
fun HtmlText(
    htmlFormattedText: Spanned,
    textSize: Float,
    modifier: Modifier = Modifier
) {
    AndroidView(
        modifier = modifier,
        factory = { context ->
            TextView(context).apply {
                setTextSize(TypedValue.COMPLEX_UNIT_SP, textSize)
            }
        },
        update = { it.text = htmlFormattedText }
    )
}
