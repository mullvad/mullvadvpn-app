package net.mullvad.mullvadvpn.util

import android.content.res.AssetManager
import android.util.Log
import java.io.IOException

private const val CHANGELOG_FILE = "en-US/default.txt"
private const val EMPTY_DEFAULT_STRING_WHEN_UNABLE_TO_READ_CHANGELOG = ""

class ChangelogDataProvider(var assets: AssetManager) : IChangelogDataProvider {
    override fun getChangelog(): String {
        return try {
            assets.open(CHANGELOG_FILE).bufferedReader().use { it.readText() }
        } catch (ex: IOException) {
            Log.e("mullvad", "Unable to read bundled changelog file.")
            EMPTY_DEFAULT_STRING_WHEN_UNABLE_TO_READ_CHANGELOG
        }
    }
}

interface IChangelogDataProvider {
    fun getChangelog(): String
}
