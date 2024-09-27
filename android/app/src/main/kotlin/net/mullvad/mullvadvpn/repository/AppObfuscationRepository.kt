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
import net.mullvad.mullvadvpn.ui.MainActivity
import net.mullvad.mullvadvpn.ui.obfuscation.MainActivityAltBrowser
import net.mullvad.mullvadvpn.ui.obfuscation.MainActivityAltGame
import net.mullvad.mullvadvpn.ui.obfuscation.MainActivityAltNinja
import net.mullvad.mullvadvpn.ui.obfuscation.MainActivityAltNotes
import net.mullvad.mullvadvpn.ui.obfuscation.MainActivityAltWeather

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

    private fun AppObfuscation.toComponentName() = ComponentName(packageName, clazz.name)
}

enum class AppObfuscation(
    val clazz: Class<*>,
    val iconId: Int,
    val bannerId: Int,
    val labelId: Int,
) {
    DEFAULT(MainActivity::class.java, R.mipmap.ic_launcher, R.mipmap.ic_banner, R.string.app_name),
    GAME(
        MainActivityAltGame::class.java,
        R.mipmap.ic_launcher_game,
        R.mipmap.ic_banner_game,
        R.string.app_name_game,
    ),
    NINJA(
        MainActivityAltNinja::class.java,
        R.mipmap.ic_launcher_ninja,
        R.mipmap.ic_banner_ninja,
        R.string.app_name_ninja,
    ),
    WEATHER(
        MainActivityAltWeather::class.java,
        R.mipmap.ic_launcher_weather,
        R.mipmap.ic_banner_weather,
        R.string.app_name_weather,
    ),
    NOTES(
        MainActivityAltNotes::class.java,
        R.mipmap.ic_launcher_notes,
        R.mipmap.ic_banner_notes,
        R.string.app_name_notes,
    ),
    BROWSER(
        MainActivityAltBrowser::class.java,
        R.mipmap.ic_launcher_browser,
        R.mipmap.ic_banner_browser,
        R.string.app_name_browser,
    ),
}
