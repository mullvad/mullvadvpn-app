package net.mullvad.mullvadvpn.test.common.extension

import androidx.test.uiautomator.By
import androidx.test.uiautomator.BySelector
import androidx.test.uiautomator.StaleObjectException
import androidx.test.uiautomator.UiDevice
import androidx.test.uiautomator.UiObject2
import androidx.test.uiautomator.UiObject2Condition
import androidx.test.uiautomator.UiObjectNotFoundException
import androidx.test.uiautomator.Until
import androidx.test.uiautomator.waitForAppToBeVisible
import co.touchlab.kermit.Logger
import java.lang.Thread.sleep
import java.util.regex.Pattern
import net.mullvad.mullvadvpn.test.common.constant.DEFAULT_TIMEOUT
import net.mullvad.mullvadvpn.test.common.constant.LONG_TIMEOUT

fun UiDevice.findObjectByCaseInsensitiveText(text: String): UiObject2 {
    return findObjectWithTimeout(By.text(Pattern.compile(text, Pattern.CASE_INSENSITIVE)))
}

fun UiDevice.hasObjectWithTimeout(selector: BySelector, timeout: Long = DEFAULT_TIMEOUT): Boolean =
    wait(Until.hasObject(selector), timeout)

fun UiDevice.findObjectWithTimeout(
    selector: BySelector,
    timeout: Long = DEFAULT_TIMEOUT,
): UiObject2 {

    wait(Until.hasObject(selector), timeout)

    val foundObject =
        findObject(selector)
            ?: throw UiObjectNotFoundException(
                "No matches for selector within timeout ($timeout ms): $selector"
            )

    return foundObject
}

fun UiDevice.expectObjectToDisappearWithTimeout(
    selector: BySelector,
    timeout: Long = DEFAULT_TIMEOUT,
) {
    val sleepInterval = 100L
    val startTime = System.currentTimeMillis()

    while (null != findObject(selector)) {
        val elapsedTime = System.currentTimeMillis() - startTime
        if (elapsedTime > timeout) {
            error("Object is still visible after timeout")
        }
        sleep(sleepInterval)
    }
}

fun UiDevice.clickObjectAwaitCondition(
    selector: BySelector,
    condition: UiObject2Condition<Boolean>,
    timeout: Long = LONG_TIMEOUT,
) {
    var foundObject = findObjectWithTimeout(selector, timeout)
    foundObject.click()

    val retryCount = 3
    repeat(retryCount) {
        try {
            val wasChecked = foundObject.wait(condition, timeout)
            require(wasChecked) {
                "UiObject2 did not become match condition within timeout $timeout ms"
            }
            return
        } catch (_: StaleObjectException) {
            Logger.e("Caught StaleObjectException - retrying")
            foundObject = findObjectWithTimeout(selector, timeout)
        }
    }
    error("Exceeded maximum StaleObjectException count ($retryCount)")
}

fun UiDevice.clickObjectAwaitIsChecked(selector: BySelector, timeout: Long = LONG_TIMEOUT) {
    clickObjectAwaitCondition(
        selector = selector,
        condition = Until.checked(true),
        timeout = timeout,
    )
}

fun UiDevice.pressBackTwice() {
    pressBack()
    pressBack()
}

fun UiObject2.findObjectWithTimeout(
    selector: BySelector,
    timeout: Long = DEFAULT_TIMEOUT,
): UiObject2 {

    wait(Until.hasObject(selector), timeout)

    return findObject(selector)
        ?: throw UiObjectNotFoundException(
            "No matches for selector within timeout ($timeout ms): $selector"
        )
}

fun UiObject2.findAncestor(selector: BySelector): UiObject2 {
    val p = parent ?: throw UiObjectNotFoundException("No ancestor matches selector: $selector")

    return p.findObject(selector) ?: p.findAncestor(selector)
}

fun UiDevice.acceptVpnPermissionDialog() {
    findObjectWithTimeout(By.text("Connection request"))
    // Accept Creating the VPN Permission profile
    findObjectWithTimeout(By.text("OK")).click()
    waitForAppToBeVisible(currentPackageName)
}
