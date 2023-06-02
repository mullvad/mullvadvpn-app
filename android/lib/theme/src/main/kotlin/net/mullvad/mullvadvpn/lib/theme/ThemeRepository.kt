package net.mullvad.mullvadvpn.lib.theme

import androidx.datastore.core.DataStore
import androidx.datastore.preferences.core.Preferences
import androidx.datastore.preferences.core.booleanPreferencesKey
import androidx.datastore.preferences.core.edit
import androidx.datastore.preferences.core.stringPreferencesKey
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn

class ThemeRepository(
    private val dataStore: DataStore<Preferences>,
    private val dispatcher: CoroutineDispatcher = Dispatchers.IO
) {
    fun useMaterialYouTheme(): StateFlow<Boolean> =
        dataStore.data
            .map { it[booleanPreferencesKey(USE_MATERIAL_YOU_THEME)] ?: false }
            .stateIn(CoroutineScope(dispatcher), SharingStarted.Eagerly, false)

    suspend fun setUseMaterialYouTheme(useMaterialYouTheme: Boolean) {
        dataStore.edit { it[booleanPreferencesKey(USE_MATERIAL_YOU_THEME)] = useMaterialYouTheme }
    }

    fun useDarkTheme(): StateFlow<DarkThemeState> =
        dataStore.data
            .map { it[stringPreferencesKey(USE_DARK_THEME)].toDarkThemeState() }
            .stateIn(CoroutineScope(dispatcher), SharingStarted.Eagerly, DarkThemeState.ON)

    suspend fun setUseDarkTheme(useDarkTheme: DarkThemeState) {
        dataStore.edit { it[stringPreferencesKey(USE_DARK_THEME)] = useDarkTheme.name }
    }

    private fun String?.toDarkThemeState(): DarkThemeState =
        when (this) {
            DarkThemeState.SYSTEM.name -> DarkThemeState.SYSTEM
            DarkThemeState.ON.name -> DarkThemeState.ON
            DarkThemeState.OFF.name -> DarkThemeState.OFF
            else -> DarkThemeState.ON
        }

    companion object {
        private const val USE_MATERIAL_YOU_THEME = "use_material_you_theme"
        private const val USE_DARK_THEME = "use_dark_theme"
    }
}
