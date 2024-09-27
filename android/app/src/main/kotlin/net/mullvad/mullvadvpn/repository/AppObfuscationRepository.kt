package net.mullvad.mullvadvpn.repository

import android.content.ComponentName
import android.content.pm.PackageManager
import android.content.pm.PackageManager.COMPONENT_ENABLED_STATE_DEFAULT
import android.content.pm.PackageManager.COMPONENT_ENABLED_STATE_DISABLED
import android.content.pm.PackageManager.COMPONENT_ENABLED_STATE_DISABLED_UNTIL_USED
import android.content.pm.PackageManager.COMPONENT_ENABLED_STATE_DISABLED_USER
import android.content.pm.PackageManager.COMPONENT_ENABLED_STATE_ENABLED
import android.content.pm.PackageManager.DONT_KILL_APP
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import net.mullvad.mullvadvpn.R

class AppObfuscationRepository(
    private val packageManager: PackageManager,
    private val obfuscationComponents: List<ComponentName>,
) {
    private val _currentAppObfuscation = MutableStateFlow(getObfuscation())
    val currentAppObfuscation: StateFlow<AppObfuscation> = _currentAppObfuscation

    val availableObfuscations: StateFlow<List<AppObfuscation>> =
        MutableStateFlow(obfuscationComponents.map { it.toAppObfuscation() })

    fun setAppObfuscation(appObfuscation: AppObfuscation) {
        obfuscationComponents.forEach {
            packageManager.setComponentEnabledSetting(
                it,
                COMPONENT_ENABLED_STATE_DISABLED,
                DONT_KILL_APP,
            )
        }
        packageManager.setComponentEnabledSetting(
            appObfuscation.toComponentName(),
            COMPONENT_ENABLED_STATE_ENABLED,
            DONT_KILL_APP,
        )

        _currentAppObfuscation.value = appObfuscation
    }

    private fun getObfuscation(): AppObfuscation =
        obfuscationComponents
            .filter { packageManager.isComponentEnabled(it) }
            .map { it.toAppObfuscation() }
            .first()

    private fun ComponentName.toAppObfuscation(): AppObfuscation =
        AppObfuscation.entries.first { this.shortClassName == it.path }

    private fun AppObfuscation.toComponentName(): ComponentName =
        obfuscationComponents.first { it.shortClassName == this.path }

    private fun PackageManager.isComponentEnabled(componentName: ComponentName): Boolean =
        when (this.getComponentEnabledSetting(componentName)) {
            COMPONENT_ENABLED_STATE_DEFAULT ->
                componentName.shortClassName == AppObfuscation.DEFAULT.path
            COMPONENT_ENABLED_STATE_ENABLED -> true
            COMPONENT_ENABLED_STATE_DISABLED -> false
            COMPONENT_ENABLED_STATE_DISABLED_USER,
            COMPONENT_ENABLED_STATE_DISABLED_UNTIL_USED ->
                error("Enabled setting only applicable for application")
            else -> error("Unknown component enabled setting")
        }
}

enum class AppObfuscation(val path: String, val iconId: Int, val labelId: Int) {
    DEFAULT(".ui.obfuscation.MainActivityDefault", R.drawable.icon_android, R.string.app_name),
    GAME(".ui.obfuscation.MainActivityAltGame", R.drawable.game_preview, R.string.app_name_game),
    NINJA(
        ".ui.obfuscation.MainActivityAltNinja",
        R.drawable.ninja_preview,
        R.string.app_name_ninja,
    ),
    BROWSER(
        ".ui.obfuscation.MainActivityAltBrowser",
        R.drawable.browser_preview,
        R.string.app_name_browser,
    ),
    NEWS(".ui.obfuscation.MainActivityAltNews", R.drawable.news_preview, R.string.app_name_news),
    WEATHER(
        ".ui.obfuscation.MainActivityAltWeather",
        R.drawable.weather_preview,
        R.string.app_name_weather,
    ),
    NOTES(
        ".ui.obfuscation.MainActivityAltNotes",
        R.drawable.notes_preview,
        R.string.app_name_notes,
    ),
    NIGHT_BROWSER(
        ".ui.obfuscation.MainActivityAltBrowserNight",
        R.drawable.browser_night_preview,
        R.string.app_name_browser,
    ),
}
