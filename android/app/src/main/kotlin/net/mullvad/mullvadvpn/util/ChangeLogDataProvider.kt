package net.mullvad.mullvadvpn.util

import android.content.res.AssetManager
import android.util.Log
import androidx.core.content.PackageManagerCompat.LOG_TAG
import java.io.IOException
import java.io.InputStream
import net.mullvad.mullvadvpn.repository.IChangeLogDataProvider

private const val CHANGES_FILE = "en-US/default.txt"

class ChangeLogDataProvider(var assets: AssetManager) : IChangeLogDataProvider {
    override fun getChangeLog(): String {
        return try {
            val inputStream: InputStream = assets.open(CHANGES_FILE)
            val size: Int = inputStream.available()
            val buffer = ByteArray(size)
            inputStream.read(buffer)
            String(buffer)
        } catch (ex: IOException) {
            Log.d(LOG_TAG, "Error capturing screenshot: " + ex.message)
            ""
        }
    }
}
