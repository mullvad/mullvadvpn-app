package net.mullvad.mullvadvpn.test.e2e.interactor

import android.content.ComponentName
import android.content.Context
import android.content.Intent
import androidx.test.uiautomator.By
import androidx.test.uiautomator.UiDevice
import net.mullvad.mullvadvpn.test.common.extension.findObjectByCaseInsensitiveText

class SystemSettingsInteractor(
    private val uiDevice: UiDevice,
    private val context: Context
) {
    fun openVpnSettings() {
        val intent = Intent("com.intent.MAIN").apply {
            addFlags(Intent.FLAG_ACTIVITY_CLEAR_TASK)
            addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
        }
        intent.component = ComponentName.unflattenFromString(
            "com.android.settings/.Settings\$VpnSettingsActivity"
        )
        context.startActivity(intent)
        Thread.sleep(1000)
    }

    fun removeAllVpnPermissions() {
        openVpnSettings()
        uiDevice.findObjects(By.descContains("Settings")).forEach {
            it.click()
            Thread.sleep(1000)
            uiDevice.findObjectByCaseInsensitiveText("forget vpn").click()
            Thread.sleep(1000)
            uiDevice.findObjectByCaseInsensitiveText("forget").click()
        }
    }
}
