package net.mullvad.mullvadvpn.util

import android.content.res.AssetManager
import android.util.Log
import java.io.IOException

private const val CHANGELOG_FILE = "en-US/default.txt"

class ChangelogDataProvider(var assets: AssetManager) : IChangelogDataProvider {
    override fun getChangelog(): String {
        return try {
            assets.open(CHANGELOG_FILE).bufferedReader().use { it.readText() }
        } catch (ex: IOException) {
            Log.e("mullvad", "Unable to read bundled changelog file.")
            ""
        }
    }
}

interface IChangelogDataProvider {
    fun getChangelog(): String
}
