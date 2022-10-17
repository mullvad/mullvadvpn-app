package net.mullvad.mullvadvpn.repository

import android.content.SharedPreferences
import net.mullvad.mullvadvpn.util.IChangelogDataProvider

private const val LAST_SHOWED_CHANGELOG_VERSION_CODE = "last_showed_changelog_version_code"

class AppChangesRepository(
    private val preferences: SharedPreferences,
    private val dataProvider: IChangelogDataProvider
) {
    fun getVersionCodeOfMostRecentChangelogShowed(): Int {
        return preferences.getInt(LAST_SHOWED_CHANGELOG_VERSION_CODE, -1)
    }

    fun setVersionCodeOfMostRecentChangelogShowed(versionCode: Int) =
        preferences.edit().putInt(LAST_SHOWED_CHANGELOG_VERSION_CODE, versionCode).apply()

    fun getLastVersionChanges(): List<String> {
        return dataProvider.getChangelog().split('\n').filter { it.isNotEmpty() }
    }
}
