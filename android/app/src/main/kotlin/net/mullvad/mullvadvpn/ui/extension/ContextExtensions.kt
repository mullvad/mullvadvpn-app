package net.mullvad.mullvadvpn.ui.extension

import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context

fun Context.copyToClipboard(content: String, clipboardLabel: String) {
    val clipboard = getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
    val clipData = ClipData.newPlainText(clipboardLabel, content)
    clipboard.setPrimaryClip(clipData)
}
