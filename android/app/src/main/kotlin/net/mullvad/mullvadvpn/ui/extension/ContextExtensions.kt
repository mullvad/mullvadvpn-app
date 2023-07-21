package net.mullvad.mullvadvpn.ui.extension

import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import androidx.fragment.app.Fragment
import net.mullvad.mullvadvpn.ui.MainActivity

fun Fragment.requireMainActivity(): MainActivity {
    return if (this.activity is MainActivity) {
        this.activity as MainActivity
    } else {
        throw IllegalStateException(
            "Fragment $this not attached to ${MainActivity::class.simpleName}."
        )
    }
}

fun Context.copyToClipboard(content: String, clipboardLabel: String) {
    val clipboard = getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
    val clipData = ClipData.newPlainText(clipboardLabel, content)
    clipboard.setPrimaryClip(clipData)
}
