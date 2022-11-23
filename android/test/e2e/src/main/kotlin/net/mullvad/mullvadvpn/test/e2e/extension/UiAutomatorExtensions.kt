package net.mullvad.mullvadvpn.test.e2e.extension

import androidx.test.uiautomator.By
import androidx.test.uiautomator.BySelector
import androidx.test.uiautomator.UiDevice
import androidx.test.uiautomator.UiObject2
import androidx.test.uiautomator.Until
import java.util.regex.Pattern
import net.mullvad.mullvadvpn.test.e2e.constant.DEFAULT_INTERACTION_TIMEOUT

fun UiDevice.findObjectByCaseInsensitiveText(text: String): UiObject2 {
    return findObjectWithTimeout(By.text(Pattern.compile(text, Pattern.CASE_INSENSITIVE)))
}

fun UiObject2.findObjectByCaseInsensitiveText(text: String): UiObject2 {
    return findObjectWithTimeout(By.text(Pattern.compile(text, Pattern.CASE_INSENSITIVE)))
}

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

fun UiObject2.findObjectWithTimeout(
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
