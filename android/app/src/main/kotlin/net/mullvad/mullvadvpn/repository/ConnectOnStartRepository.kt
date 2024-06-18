package net.mullvad.mullvadvpn.repository

import android.content.ComponentName
import android.content.pm.PackageManager
import android.content.pm.PackageManager.COMPONENT_ENABLED_STATE_DEFAULT
import android.content.pm.PackageManager.COMPONENT_ENABLED_STATE_DISABLED
import android.content.pm.PackageManager.COMPONENT_ENABLED_STATE_ENABLED
import android.content.pm.PackageManager.DONT_KILL_APP
import kotlinx.coroutines.flow.MutableStateFlow

class ConnectOnStartRepository(
    private val packageManager: PackageManager,
    private val bootCompletedComponentName: ComponentName
) {
    val connectOnStart = MutableStateFlow(isConnectOnStart())

    fun setConnectOnStart(connect: Boolean) {
        packageManager.setComponentEnabledSetting(
            bootCompletedComponentName,
            if (connect) {
                COMPONENT_ENABLED_STATE_ENABLED
            } else {
                COMPONENT_ENABLED_STATE_DISABLED
            },
            DONT_KILL_APP
        )

        connectOnStart.value = isConnectOnStart()
    }

    private fun isConnectOnStart(): Boolean =
        when (packageManager.getComponentEnabledSetting(bootCompletedComponentName)) {
            COMPONENT_ENABLED_STATE_DEFAULT -> BOOT_COMPLETED_DEFAULT_STATE
            COMPONENT_ENABLED_STATE_ENABLED -> true
            COMPONENT_ENABLED_STATE_DISABLED -> false
            else -> error("Unknown component enabled setting")
        }

    companion object {
        private const val BOOT_COMPLETED_DEFAULT_STATE = false
    }
}
