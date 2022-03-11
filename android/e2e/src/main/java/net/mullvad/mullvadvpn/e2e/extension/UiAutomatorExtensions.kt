package net.mullvad.mullvadvpn.e2e.extension

import androidx.test.uiautomator.BySelector
import androidx.test.uiautomator.UiDevice
import androidx.test.uiautomator.UiObject2
import androidx.test.uiautomator.Until
import net.mullvad.mullvadvpn.e2e.constant.DEFAULT_INTERACTION_TIMEOUT

fun UiDevice.findObjectWithTimeout(
    selector: BySelector,
    timeout: Long = DEFAULT_INTERACTION_TIMEOUT
): UiObject2 {

    wait(
        Until.hasObject(selector),
        timeout
    )

    return try {
        findObject(selector)
    } catch (e: NullPointerException) {
        throw IllegalArgumentException(
            "No matches for selector within timeout ($timeout): $selector"
        )
    }
}
