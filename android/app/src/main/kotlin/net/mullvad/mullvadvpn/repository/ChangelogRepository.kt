package net.mullvad.mullvadvpn.repository

import android.content.SharedPreferences
import net.mullvad.mullvadvpn.util.IChangelogDataProvider
import net.mullvad.mullvadvpn.util.trimAll

private const val MISSING_VERSION_CODE = -1
private const val NEWLINE_CHAR = '\n'
private const val BULLET_POINT_CHAR = '-'
private const val LAST_SHOWED_CHANGELOG_VERSION_CODE = "last_showed_changelog_version_code"

class ChangelogRepository(
    private val preferences: SharedPreferences,
    private val dataProvider: IChangelogDataProvider,
) {
    fun getVersionCodeOfMostRecentChangelogShowed(): Int {
        return preferences.getInt(LAST_SHOWED_CHANGELOG_VERSION_CODE, MISSING_VERSION_CODE)
    }

    fun setVersionCodeOfMostRecentChangelogShowed(versionCode: Int) =
        preferences.edit().putInt(LAST_SHOWED_CHANGELOG_VERSION_CODE, versionCode).apply()

    fun getLastVersionChanges(): List<String> =
        // Prepend with a new line char so each entry consists of NEWLINE_CHAR + BULLET_POINT_CHAR
        (NEWLINE_CHAR + dataProvider.getChangelog())
            .split(NEWLINE_CHAR.toString() + BULLET_POINT_CHAR)
            .trimAll()
            .filter { it.isNotEmpty() }
}
