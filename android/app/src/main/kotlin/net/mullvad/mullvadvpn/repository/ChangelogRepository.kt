package net.mullvad.mullvadvpn.repository

import android.content.SharedPreferences
import net.mullvad.mullvadvpn.util.IChangelogDataProvider

private const val MISSING_VERSION_CODE = -1
private const val NEWLINE_CHAR = '\n'
private const val LAST_SHOWED_CHANGELOG_VERSION_CODE = "last_showed_changelog_version_code"

class ChangelogRepository(
    private val preferences: SharedPreferences,
    private val dataProvider: IChangelogDataProvider
) {
    fun getVersionCodeOfMostRecentChangelogShowed(): Int {
        return preferences.getInt(LAST_SHOWED_CHANGELOG_VERSION_CODE, MISSING_VERSION_CODE)
    }

    fun setVersionCodeOfMostRecentChangelogShowed(versionCode: Int) =
        preferences.edit().putInt(LAST_SHOWED_CHANGELOG_VERSION_CODE, versionCode).apply()

    fun getLastVersionChanges(): List<String> {
        return dataProvider.getChangelog().split(NEWLINE_CHAR).filter { it.isNotEmpty() }
    }
}
