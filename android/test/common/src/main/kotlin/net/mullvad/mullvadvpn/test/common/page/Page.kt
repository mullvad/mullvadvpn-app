package net.mullvad.mullvadvpn.test.common.page

import androidx.test.platform.app.InstrumentationRegistry
import androidx.test.uiautomator.UiDevice

sealed class Page {
    val uiDevice = UiDevice.getInstance(InstrumentationRegistry.getInstrumentation())

    abstract fun assertIsDisplayed()
}

inline fun <reified T : Page> on(scope: T.() -> Unit = {}) {
    val page = T::class.java.getConstructor().newInstance()
    page.assertIsDisplayed()
    return page.scope()
}
