package net.mullvad.mullvadvpn.repository

import android.content.SharedPreferences
import android.content.res.AssetManager
import java.io.IOException
import java.io.InputStream
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Dispatchers
import net.mullvad.mullvadvpn.BuildConfig

private const val KEY_CHANGES_SHOWED = BuildConfig.VERSION_NAME
private const val CHANGES_FILE = "en-US/default.txt"

enum class ChangeLogState {
    ShouldShow,
    AlreadyShowed,
    Unknown
}

class AppChangesRepository(
    private val preferences: SharedPreferences,
    private val assets: AssetManager,
    private val dispatcher: CoroutineDispatcher = Dispatchers.IO
) {

    fun resetShouldShowLastChanges() {
        preferences.edit().putBoolean(KEY_CHANGES_SHOWED, false).apply()
    }

    fun shouldShowLastChanges(): Boolean {
        return preferences.getBoolean(KEY_CHANGES_SHOWED, false).not()
    }

    fun setShowedLastChanges() =
        preferences.edit().putBoolean(KEY_CHANGES_SHOWED, true).apply()

    fun getLastVersionChanges(): List<String> {
        return try {
            val inputStream: InputStream = assets.open(CHANGES_FILE)
            val size: Int = inputStream.available()
            val buffer = ByteArray(size)
            inputStream.read(buffer)
            String(buffer).split('\n')
                .filter { !it.isNullOrEmpty() }
                .map { "$it" }
        } catch (e: IOException) {
            e.printStackTrace()
            ArrayList()
        }
    }
}
