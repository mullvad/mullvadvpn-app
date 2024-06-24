package net.mullvad.mullvadvpn.util

import android.text.Html
import androidx.core.text.HtmlCompat

fun String.appendHideNavOnPlayBuild(isPlayBuild: Boolean): String =
    if (isPlayBuild) {
        "$this?hide_nav"
    } else {
        this
    }

fun String.removeHtmlTags(): String =
    Html.fromHtml(this, HtmlCompat.FROM_HTML_MODE_LEGACY).toString()

fun List<String>.trimAll() = map { it.trim() }
