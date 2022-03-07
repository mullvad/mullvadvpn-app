package net.mullvad.mullvadvpn.e2e.interactor

import androidx.test.uiautomator.By
import androidx.test.uiautomator.UiDevice
import androidx.test.uiautomator.UiObject2
import net.mullvad.mullvadvpn.e2e.constant.NOTIFICATION_STACK_ID
import net.mullvad.mullvadvpn.e2e.constant.SYSTEM_UI_PACKAGE
import net.mullvad.mullvadvpn.e2e.extension.findObjectWithTimeout

class NotificationStackInteractor(
    private val device: UiDevice
) {
    fun ensureNotificationExpandedByTitle(notificationTitle: String) {
        findNotificationStack().findObjectWithTimeout(By.text(notificationTitle)).click()
    }

    fun clickNotificationActionButtonByText(buttonText: String) {
        findNotificationStack().findObjectWithTimeout(By.text(buttonText)).click()
    }

    private fun findNotificationStack(): UiObject2 {
        return device.findObjectWithTimeout(
            By
                .pkg(SYSTEM_UI_PACKAGE)
                .res(NOTIFICATION_STACK_ID)
        )
    }
}
