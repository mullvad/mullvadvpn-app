package net.mullvad.mullvadvpn.lib.repository

import android.content.res.AssetManager
import co.touchlab.kermit.Logger
import java.io.IOException

class ChangelogDataProvider(private var assets: AssetManager) {
    fun getChangelog(): String {
        return try {
            assets.open(CHANGELOG_FILE).bufferedReader().use { it.readText() }
        } catch (ex: IOException) {
            Logger.Companion.e("Unable to read bundled changelog file.")
            EMPTY_DEFAULT_STRING_WHEN_UNABLE_TO_READ_CHANGELOG
        }
    }

    companion object {
        private const val CHANGELOG_FILE = "en-US/default.txt"
        private const val EMPTY_DEFAULT_STRING_WHEN_UNABLE_TO_READ_CHANGELOG = ""
    }
}
