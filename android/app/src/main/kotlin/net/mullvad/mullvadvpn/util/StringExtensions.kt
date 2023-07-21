package net.mullvad.mullvadvpn.util

import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.widget.Toast
import net.mullvad.mullvadvpn.BuildConfig
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.constant.BuildTypes
import org.joda.time.DateTime
import org.joda.time.format.DateTimeFormat

private const val EXPIRY_FORMAT = "YYYY-MM-dd HH:mm:ss z"
private const val BIG_DOT_CHAR = "●"
private const val SPACE_CHAR = ' '

fun String.capitalizeFirstCharOfEachWord(): String {
    return split(" ")
        .joinToString(" ") { word -> word.replaceFirstChar { firstChar -> firstChar.uppercase() } }
        .trimEnd()
}

fun String.parseAsDateTime(): DateTime? {
    return try {
        DateTime.parse(this, DateTimeFormat.forPattern(EXPIRY_FORMAT))
    } catch (ex: Exception) {
        null
    }
}

fun String.appendHideNavOnReleaseBuild(): String =
    if (BuildTypes.RELEASE == BuildConfig.BUILD_TYPE) {
        "$this?hide_nav"
    } else {
        this
    }

fun String.copyToClipboard(
    context: Context,
    clipboardLabel: String,
    copiedToastMessage: String? = null
) {
    val clipboard = context.getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
    val clipData = ClipData.newPlainText(clipboardLabel, this)
    val toastMessage = copiedToastMessage ?: context.getString(R.string.copied_to_clipboard)

    clipboard.setPrimaryClip(clipData)

    Toast.makeText(context, toastMessage, Toast.LENGTH_SHORT).show()
}

fun String.groupWithSpaces(groupCharSize: Int = 4): String {
    return fold(StringBuilder()) { formattedText, nextDigit ->
            if ((formattedText.length % (groupCharSize + 1)) == groupCharSize) {
                formattedText.append(SPACE_CHAR)
            }
            formattedText.append(nextDigit)
        }
        .toString()
}

fun String.groupPasswordModeWithSpaces(groupCharSize: Int = 4): String {
    return BIG_DOT_CHAR.repeat(this.length).groupWithSpaces(groupCharSize)
}
