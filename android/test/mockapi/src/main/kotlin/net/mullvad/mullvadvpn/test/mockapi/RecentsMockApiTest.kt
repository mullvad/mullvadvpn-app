package net.mullvad.mullvadvpn.test.mockapi

import androidx.test.uiautomator.By
import androidx.test.uiautomator.Direction
import androidx.test.uiautomator.waitForStableInActiveWindow
import java.time.ZonedDateTime
import net.mullvad.mullvadvpn.lib.ui.tag.HOP_SELECTOR_ENTRY_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.RECENT_CELL_TEST_TAG
import net.mullvad.mullvadvpn.test.common.extension.acceptVpnPermissionDialog
import net.mullvad.mullvadvpn.test.common.extension.findObjectWithTimeout
import net.mullvad.mullvadvpn.test.common.extension.findOneOrMoreObjectsWithTimeout
import net.mullvad.mullvadvpn.test.common.page.ConnectPage
import net.mullvad.mullvadvpn.test.common.page.SelectLocationPage
import net.mullvad.mullvadvpn.test.common.page.enableMultihopStory
import net.mullvad.mullvadvpn.test.common.page.on
import net.mullvad.mullvadvpn.test.common.rule.ForgetAllVpnAppsInSettingsTestRule
import net.mullvad.mullvadvpn.test.mockapi.constant.DEFAULT_DEVICE_LIST
import net.mullvad.mullvadvpn.test.mockapi.constant.DUMMY_DEVICE_NAME_2
import net.mullvad.mullvadvpn.test.mockapi.constant.DUMMY_ID_2
import org.junit.jupiter.api.Assertions.assertFalse
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.assertNotNull
import org.junit.jupiter.api.extension.RegisterExtension

class RecentsMockApiTest : MockApiTest() {

    @RegisterExtension
    @JvmField
    val forgetAllVpnAppsInSettingsTestRule = ForgetAllVpnAppsInSettingsTestRule()

    val validAccountNumber = "1234123412341234"

    @BeforeEach
    fun setupDispatcher() {
        apiDispatcher.apply {
            expectedAccountNumber = validAccountNumber
            accountExpiry = ZonedDateTime.now().plusMonths(1)
            devices = DEFAULT_DEVICE_LIST.toMutableMap()
            devicePendingToGetCreated = DUMMY_ID_2 to DUMMY_DEVICE_NAME_2
        }
    }

    @Test
    fun testRecentsEnableDisable() {

        app.launchAndLogIn(validAccountNumber)

        on<ConnectPage> { clickSelectLocation() }

        on<SelectLocationPage> {
            // Check that the 'Recents' header is visible.
            // Recents should be visible even though we have not connected yet.
            device.findObjectWithTimeout(By.text("Recents"))

            clickMenuButton()
            clickDisableRecentsButton()

            // Check that 'Recents' header is no longer visible.
            assertFalse(device.hasObject(By.text("Recents")))

            // Connect to a relay we know is visible.
            device.findObjectWithTimeout(By.text("Albania")).click()
            device.acceptVpnPermissionDialog()
        }

        on<ConnectPage> { clickSelectLocation() }

        on<SelectLocationPage> {
            clickMenuButton()
            clickEnableRecentsButton()

            scrollUntilText("Recents", Direction.UP)

            val recentCell = device.findObjectWithTimeout(By.res(RECENT_CELL_TEST_TAG))

            // Check that the previously selected relay is in the recents list
            assertNotNull(recentCell.findObject(By.text("Albania")))
        }
    }

    @Test
    fun testRecentsWithMultihop() {

        app.launchAndLogIn(validAccountNumber)

        // Enable Multihop
        on<ConnectPage> {
            enableMultihopStory()

            clickConnect()
            device.acceptVpnPermissionDialog(ignoreNotFound = true)

            clickSelectLocation()
        }

        on<SelectLocationPage> {
            // Check that we have a recent entry
            val recentCell = device.findObjectWithTimeout(By.res(RECENT_CELL_TEST_TAG))
            assertNotNull(recentCell.findObject(By.text("Sweden")))

            // Set exit relay to Albania
            device.findObjectWithTimeout(By.text("Albania")).click()
        }

        on<ConnectPage> {
            clickSelectLocation()

            // Check that the exit recent lists contain Sweden and Albania
            device.findOneOrMoreObjectsWithTimeout(By.res(RECENT_CELL_TEST_TAG)).forEach {
                assert(
                    it.findObject(By.text("Sweden")) != null ||
                        it.findObject(By.text("Albania")) != null
                )
            }

            val entry = device.findObjectWithTimeout(By.res(HOP_SELECTOR_ENTRY_TEST_TAG))

            entry.click()

            device.waitForStableInActiveWindow()

            // Check that the entry recent list only contains Sweden
            device.findOneOrMoreObjectsWithTimeout(By.res(RECENT_CELL_TEST_TAG)).forEach {
                assertNotNull(it.findObject(By.text("Sweden")))
            }
        }
    }
}
