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

class AutoStartAndConnectOnBootRepository(
    private val packageManager: PackageManager,
    private val bootCompletedComponentName: ComponentName
) {
    val autoStartAndConnectOnBoot = MutableStateFlow(isAutoStartAndConnectOnBoot())

    fun setAutoStartAndConnectOnBoot(connect: Boolean) {
        packageManager.setComponentEnabledSetting(
            bootCompletedComponentName,
            if (connect) {
                COMPONENT_ENABLED_STATE_ENABLED
            } else {
                COMPONENT_ENABLED_STATE_DISABLED
            },
            DONT_KILL_APP
        )

        autoStartAndConnectOnBoot.value = isAutoStartAndConnectOnBoot()
    }

    private fun isAutoStartAndConnectOnBoot(): Boolean =
        when (packageManager.getComponentEnabledSetting(bootCompletedComponentName)) {
            COMPONENT_ENABLED_STATE_DEFAULT -> BOOT_COMPLETED_DEFAULT_STATE
            COMPONENT_ENABLED_STATE_ENABLED -> true
            COMPONENT_ENABLED_STATE_DISABLED -> false
            COMPONENT_ENABLED_STATE_DISABLED_USER,
            COMPONENT_ENABLED_STATE_DISABLED_UNTIL_USED ->
                error("Enabled setting only applicable for application")
            else -> error("Unknown component enabled setting")
        }

    companion object {
        private const val BOOT_COMPLETED_DEFAULT_STATE = false
    }
}
