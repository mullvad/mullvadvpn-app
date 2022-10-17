package net.mullvad.mullvadvpn.util

import android.content.res.AssetManager
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
        } catch (e: IOException) {
            e.printStackTrace()
            ""
        }
    }
}
