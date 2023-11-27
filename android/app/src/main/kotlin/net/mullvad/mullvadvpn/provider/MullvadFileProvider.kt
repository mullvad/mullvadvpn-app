package net.mullvad.mullvadvpn.provider

import android.content.Context
import android.content.Intent
import android.net.Uri
import androidx.core.content.FileProvider
import net.mullvad.mullvadvpn.BuildConfig
import java.io.File
import net.mullvad.mullvadvpn.R
import org.joda.time.DateTime
import org.joda.time.format.ISODateTimeFormat

// https://developer.android.com/reference/androidx/core/content/FileProvider
// From link: It is possible to use FileProvider directly instead of extending it. However, this is
// not reliable and will causes crashes on some devices.
class MullvadFileProvider : FileProvider(R.xml.provider_paths) {
    companion object {
        fun uriForFile(context: Context, file: File): Uri {
            return getUriForFile(context, "${context.packageName}.FileProvider", file)
        }
    }
}

enum class ProviderCacheDirectory(val directoryName: String) {
    LOGS("logs")
}

fun Context.getLogsShareIntent(shareTitle: String, logContent: String): Intent {
    val fileName = createShareLogFileName()
    val cacheFile = createCacheFile(ProviderCacheDirectory.LOGS, fileName)
    cacheFile.writeText(logContent)
    val logsUri = MullvadFileProvider.uriForFile(this, cacheFile)

    val sendIntent: Intent =
        Intent().apply {
            action = Intent.ACTION_SEND
            type = "text/plain"
            putExtra(Intent.EXTRA_STREAM, logsUri)
            addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
        }
    return Intent.createChooser(sendIntent, null)
}

fun Context.createCacheFile(
    directory: ProviderCacheDirectory,
    fileName: String,
): File {
    // Path to log file
    val logsPath = File(cacheDir, directory.directoryName)

    // Ensure path is created
    logsPath.mkdirs()

    return File(logsPath, fileName)
}

fun createShareLogFileName(): String {
    val datetime = ISODateTimeFormat.basicOrdinalDateTimeNoMillis().print(DateTime.now())
    return "mullvad_log-${datetime}.txt"
}
