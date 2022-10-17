package net.mullvad.mullvadvpn.repository

import android.content.SharedPreferences
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Dispatchers
import net.mullvad.mullvadvpn.BuildConfig

private const val KEY_CHANGES_SHOWED = "key_changes_showed"

interface IChangeLogDataProvider {
    fun getChangeLog(): String
}

class AppChangesRepository(
    private val preferences: SharedPreferences,
    private val dataProvider: IChangeLogDataProvider,
    private val dispatcher: CoroutineDispatcher = Dispatchers.IO
) {
    fun resetShouldShowLastChanges() {
        preferences.edit().putInt(KEY_CHANGES_SHOWED, BuildConfig.VERSION_CODE).apply()
    }

    fun shouldShowLastChanges(): Boolean {
        return preferences.getInt(KEY_CHANGES_SHOWED, -1) < BuildConfig.VERSION_CODE ||
            BuildConfig.ALWAYS_SHOW_CHANGELOG
    }

    fun setShowedLastChanges() =
        preferences.edit().putInt(KEY_CHANGES_SHOWED, BuildConfig.VERSION_CODE).apply()

    fun getLastVersionChanges(): List<String> {
        return dataProvider.getChangeLog().split('\n')
            .filter { !it.isNullOrEmpty() }
            .map { "$it" }
    }
}
