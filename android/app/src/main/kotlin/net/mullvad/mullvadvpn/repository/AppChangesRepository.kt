package net.mullvad.mullvadvpn.repository

import android.content.SharedPreferences
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Dispatchers
import net.mullvad.mullvadvpn.BuildConfig

private const val KEY_CHANGES_SHOWED = BuildConfig.VERSION_NAME

enum class ChangeLogState {
    ShouldShow,
    AlreadyShowed
}

interface IChangeLogDataProvider {
    fun getChangeLog(): String
}

class AppChangesRepository(
    private val preferences: SharedPreferences,
    private val dataProvider: IChangeLogDataProvider,
    private val dispatcher: CoroutineDispatcher = Dispatchers.IO
) {

    fun resetShouldShowLastChanges() {
        preferences.edit().putBoolean(KEY_CHANGES_SHOWED, false).apply()
    }

    fun shouldShowLastChanges(): Boolean {
        return preferences.getBoolean(KEY_CHANGES_SHOWED, false)
            .not() || BuildConfig.ALWAYS_SHOW_CHANGELOG
    }

    fun setShowedLastChanges() =
        preferences.edit().putBoolean(KEY_CHANGES_SHOWED, true).apply()

    fun getLastVersionChanges(): List<String> {
        return dataProvider.getChangeLog().split('\n')
            .filter { !it.isNullOrEmpty() }
            .map { "$it" }
    }
}
