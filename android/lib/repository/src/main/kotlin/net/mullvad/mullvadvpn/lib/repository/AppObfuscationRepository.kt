package net.mullvad.mullvadvpn.lib.repository

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
import net.mullvad.mullvadvpn.lib.common.constant.MAIN_ACTIVITY_ALT_BROWSER_CLASS
import net.mullvad.mullvadvpn.lib.common.constant.MAIN_ACTIVITY_ALT_DEFAULT_CLASS
import net.mullvad.mullvadvpn.lib.common.constant.MAIN_ACTIVITY_ALT_GAME_CLASS
import net.mullvad.mullvadvpn.lib.common.constant.MAIN_ACTIVITY_ALT_NINJA_CLASS
import net.mullvad.mullvadvpn.lib.common.constant.MAIN_ACTIVITY_ALT_NOTES_CLASS
import net.mullvad.mullvadvpn.lib.common.constant.MAIN_ACTIVITY_ALT_WEATHER_CLASS
import net.mullvad.mullvadvpn.lib.common.constant.MAIN_ACTIVITY_CLASS

class AppObfuscationRepository(
    private val packageManager: PackageManager,
    private val packageName: String,
) {
    private val _currentAppObfuscation = MutableStateFlow(getObfuscation())
    val currentAppObfuscation: StateFlow<AppObfuscation> = _currentAppObfuscation

    val availableObfuscations: StateFlow<List<AppObfuscation>> =
        MutableStateFlow(AppObfuscation.entries)

    fun setAppObfuscation(appObfuscation: AppObfuscation) {
        AppObfuscation.entries.forEach {
            packageManager.setComponentEnabledSetting(
                it.toComponentName(),
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
        AppObfuscation.entries.first { packageManager.isComponentEnabled(it.toComponentName()) }

    private fun PackageManager.isComponentEnabled(componentName: ComponentName): Boolean =
        when (this.getComponentEnabledSetting(componentName)) {
            COMPONENT_ENABLED_STATE_DEFAULT ->
                componentName == AppObfuscation.DEFAULT.toComponentName()
            COMPONENT_ENABLED_STATE_ENABLED -> true
            COMPONENT_ENABLED_STATE_DISABLED -> false
            COMPONENT_ENABLED_STATE_DISABLED_USER,
            COMPONENT_ENABLED_STATE_DISABLED_UNTIL_USED ->
                error("Enabled setting only applicable for application")
            else -> error("Unknown component enabled setting")
        }

    private fun AppObfuscation.toComponentName() = ComponentName(packageName, className)
}

enum class AppObfuscation(
    val className: String,
    val iconId: Int,
    val bannerId: Int,
    val labelId: Int,
) {
    DEFAULT(
        MAIN_ACTIVITY_ALT_DEFAULT_CLASS,
        R.mipmap.ic_launcher,
        R.mipmap.ic_banner,
        R.string.app_name,
    ),
    GAME(
        MAIN_ACTIVITY_ALT_GAME_CLASS,
        R.mipmap.ic_launcher_game,
        R.mipmap.ic_banner_game,
        R.string.app_name_game,
    ),
    NINJA(
        MAIN_ACTIVITY_ALT_NINJA_CLASS,
        R.mipmap.ic_launcher_ninja,
        R.mipmap.ic_banner_ninja,
        R.string.app_name_ninja,
    ),
    WEATHER(
        MAIN_ACTIVITY_ALT_WEATHER_CLASS,
        R.mipmap.ic_launcher_weather,
        R.mipmap.ic_banner_weather,
        R.string.app_name_weather,
    ),
    NOTES(
        MAIN_ACTIVITY_ALT_NOTES_CLASS,
        R.mipmap.ic_launcher_notes,
        R.mipmap.ic_banner_notes,
        R.string.app_name_notes,
    ),
    BROWSER(
        MAIN_ACTIVITY_ALT_BROWSER_CLASS,
        R.mipmap.ic_launcher_browser,
        R.mipmap.ic_banner_browser,
        R.string.app_name_browser,
    ),
}
