package net.mullvad.mullvadvpn.e2e.interactor

import android.content.Context
import android.content.Intent
import androidx.test.uiautomator.By
import androidx.test.uiautomator.UiDevice
import androidx.test.uiautomator.Until
import net.mullvad.mullvadvpn.e2e.constant.APP_LAUNCH_TIMEOUT
import net.mullvad.mullvadvpn.e2e.constant.MULLVAD_PACKAGE

class AppInteractor(
    private val device: UiDevice,
    private val targetContext: Context
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
}
