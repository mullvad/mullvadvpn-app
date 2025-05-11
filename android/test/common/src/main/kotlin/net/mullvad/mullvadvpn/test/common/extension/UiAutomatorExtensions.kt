package net.mullvad.mullvadvpn.test.common.extension

import android.os.Build
import androidx.test.uiautomator.By
import androidx.test.uiautomator.BySelector
import androidx.test.uiautomator.StableResult
import androidx.test.uiautomator.StaleObjectException
import androidx.test.uiautomator.UiDevice
import androidx.test.uiautomator.UiObject2
import androidx.test.uiautomator.UiObject2Condition
import androidx.test.uiautomator.Until
import androidx.test.uiautomator.waitForStableInActiveWindow
import co.touchlab.kermit.Logger
import java.lang.Thread.sleep
import java.util.regex.Pattern
import net.mullvad.mullvadvpn.test.common.constant.DEFAULT_TIMEOUT
import net.mullvad.mullvadvpn.test.common.constant.LONG_TIMEOUT

fun UiDevice.findObjectByCaseInsensitiveText(text: String): UiObject2 {
    return findObjectWithTimeout(By.text(Pattern.compile(text, Pattern.CASE_INSENSITIVE)))
}

fun UiObject2.findObjectByCaseInsensitiveText(text: String): UiObject2 {
    return findObjectWithTimeout(By.text(Pattern.compile(text, Pattern.CASE_INSENSITIVE)))
}

fun UiDevice.hasObjectWithTimeout(selector: BySelector, timeout: Long = DEFAULT_TIMEOUT): Boolean =
    wait(Until.hasObject(selector), timeout)

fun UiDevice.findObjectWithTimeout(
    selector: BySelector,
    timeout: Long = DEFAULT_TIMEOUT,
): UiObject2 {

    wait(Until.hasObject(selector), timeout)

    val foundObject = findObject(selector)

    require(foundObject != null) {
        "No matches for selector within timeout ($timeout ms): $selector"
    }

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

fun UiDevice.clickAgreeOnPrivacyDisclaimer() {
    findObjectWithTimeout(By.text("Agree and continue")).click()
}

// The dialog will only be shown when there's a new version code and bundled release notes.
fun UiDevice.dismissChangelogDialogIfShown() {
    try {
        findObjectWithTimeout(By.text("Got it!")).click()
    } catch (e: IllegalArgumentException) {
        // This is OK since it means the changes dialog wasn't shown.
    }
}

fun UiDevice.clickAllowOnNotificationPermissionPromptIfApiLevel33AndAbove(
    timeout: Long = DEFAULT_TIMEOUT
) {
    if (Build.VERSION.SDK_INT < Build.VERSION_CODES.TIRAMISU) {
        // Skipping as notification permissions are not shown.
        return
    }

    val selector = By.text("Allow")

    wait(Until.hasObject(selector), timeout)

    try {
        findObjectWithTimeout(selector).click()
    } catch (e: IllegalArgumentException) {
        throw IllegalArgumentException(
            "Failed to allow notification permission within timeout ($timeout ms)"
        )
    }
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

    return try {
        findObject(selector)
    } catch (e: NullPointerException) {
        throw IllegalArgumentException(
            "No matches for selector within timeout ($timeout ms): $selector"
        )
    }
}

fun UiDevice.waitForStableInActiveWindowSafe(): StableResult? {
    return try {
        // This is a workaround for a NullPointerException that may occurs invoking waitForStable
        // and it is unable to find the current window. To avoid failing our test because unstable
        // api we try and just catch this.
        waitForStableInActiveWindow()
    } catch (e: NullPointerException) {
        Logger.e(e) { "waitForStableInActive window failed" }
        // This is the stack trace of the exception we try to grab:
        // java.lang.NullPointerException:
        // Attempt to invoke virtual method 'int
        // android.view.accessibility.AccessibilityWindowInfo.getDisplayId()' on a null object
        // reference
        //	at
        // androidx.test.uiautomator.UiDeviceExt.waitForStableInActiveWindow$lambda$3(UiDeviceExt.kt:112)

        // Since waitForStable failed we sleep for some time and hope the UI is stable by then.
        sleep(2000)
        null
    }
}
