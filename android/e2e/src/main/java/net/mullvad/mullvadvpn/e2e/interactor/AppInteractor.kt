package net.mullvad.mullvadvpn.e2e.interactor

import android.content.Context
import android.content.Intent
import androidx.test.uiautomator.By
import androidx.test.uiautomator.UiDevice
import androidx.test.uiautomator.Until
import net.mullvad.mullvadvpn.e2e.constant.APP_LAUNCH_TIMEOUT
import net.mullvad.mullvadvpn.e2e.constant.MULLVAD_PACKAGE
import net.mullvad.mullvadvpn.e2e.constant.SETTINGS_COG_ID
import net.mullvad.mullvadvpn.e2e.extension.findObjectWithTimeout

class AppInteractor(
    private val device: UiDevice,
    private val targetContext: Context,
    private val validTestAccountToken: String,
    private val invalidTestAccountToken: String
) {
    fun launch() {
        device.pressHome()
        // Wait for launcher
        device.wait(
            Until.hasObject(By.pkg(device.launcherPackageName).depth(0)),
            APP_LAUNCH_TIMEOUT
        )
        val intent =
            targetContext.packageManager.getLaunchIntentForPackage(MULLVAD_PACKAGE)?.apply {
                // Clear out any previous instances
                addFlags(Intent.FLAG_ACTIVITY_CLEAR_TASK)
            }
        targetContext.startActivity(intent)
        device.wait(
            Until.hasObject(By.pkg(MULLVAD_PACKAGE).depth(0)),
            APP_LAUNCH_TIMEOUT
        )
    }

    fun clickSettingsCog() {
        device.findObjectWithTimeout(By.res(SETTINGS_COG_ID)).click()
    }

    fun clickListItemByText(text: String) {
        device.findObjectWithTimeout(By.text(text)).click()
    }
}
