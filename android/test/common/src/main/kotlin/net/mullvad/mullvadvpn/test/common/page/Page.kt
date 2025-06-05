package net.mullvad.mullvadvpn.test.common.page

import androidx.test.platform.app.InstrumentationRegistry
import androidx.test.uiautomator.UiDevice
import net.mullvad.mullvadvpn.test.common.extension.waitForStableInActiveWindowSafe

sealed class Page {
    val uiDevice = UiDevice.getInstance(InstrumentationRegistry.getInstrumentation())

    abstract fun assertIsDisplayed()
}

inline fun <reified T : Page> on(scope: T.() -> Unit = {}) {
    val page = T::class.java.getConstructor().newInstance()
    // Wait for the screen to settle and so we don't proceed with actions too early. Otherwise, we
    // might start clicking on the screen before it is in a resumed state.
    page.uiDevice.waitForStableInActiveWindowSafe()
    page.assertIsDisplayed()

    page.scope()
}
