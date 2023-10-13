package net.mullvad.mullvadvpn

import app.cash.turbine.test
import io.mockk.MockKAnnotations
import io.mockk.every
import io.mockk.mockk
import io.mockk.unmockkAll
import kotlin.test.assertEquals
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.repository.InAppNotification
import net.mullvad.mullvadvpn.repository.InAppNotificationController
import net.mullvad.mullvadvpn.usecase.AccountExpiryNotificationUseCase
import net.mullvad.mullvadvpn.usecase.NewDeviceNotificationUseCase
import net.mullvad.mullvadvpn.usecase.TunnelStateNotificationUseCase
import net.mullvad.mullvadvpn.usecase.VersionNotificationUseCase
import net.mullvad.talpid.tunnel.ErrorState
import org.joda.time.DateTime
import org.junit.After
import org.junit.Before
import org.junit.Rule
import org.junit.Test

class InAppNotificationControllerTest {
    @get:Rule val testCoroutineRule = TestCoroutineRule()

    private lateinit var inAppNotificationController: InAppNotificationController
    private val accountExpiryNotifications = MutableStateFlow(emptyList<InAppNotification>())
    private val newDeviceNotifications = MutableStateFlow(emptyList<InAppNotification.NewDevice>())
    private val versionNotifications = MutableStateFlow(emptyList<InAppNotification>())
    private val tunnelStateNotifications = MutableStateFlow(emptyList<InAppNotification>())

    private lateinit var job: Job

    @Before
    fun setup() {
        MockKAnnotations.init(this)

        val accountExpiryNotificationUseCase: AccountExpiryNotificationUseCase = mockk()
        val newDeviceNotificationUseCase: NewDeviceNotificationUseCase = mockk()
        val versionNotificationUseCase: VersionNotificationUseCase = mockk()
        val tunnelStateNotificationUseCase: TunnelStateNotificationUseCase = mockk()
        every { accountExpiryNotificationUseCase.notifications() } returns
            accountExpiryNotifications
        every { newDeviceNotificationUseCase.notifications() } returns newDeviceNotifications
        every { versionNotificationUseCase.notifications() } returns versionNotifications
        every { tunnelStateNotificationUseCase.notifications() } returns tunnelStateNotifications
        job = Job()

        inAppNotificationController =
            InAppNotificationController(
                accountExpiryNotificationUseCase,
                newDeviceNotificationUseCase,
                versionNotificationUseCase,
                tunnelStateNotificationUseCase,
                CoroutineScope(job + testCoroutineRule.testDispatcher)
            )
    }

    @After
    fun teardown() {
        job.cancel()
        unmockkAll()
    }

    @Test
    fun `ensure all notifications have the right priority`() = runTest {
        val newDevice = InAppNotification.NewDevice("")
        newDeviceNotifications.value = listOf(newDevice)

        val errorState: ErrorState = mockk()
        val tunnelStateBlocked = InAppNotification.TunnelStateBlocked
        val tunnelStateError = InAppNotification.TunnelStateError(errorState)
        tunnelStateNotifications.value = listOf(tunnelStateBlocked, tunnelStateError)

        val unsupportedVersion = InAppNotification.UnsupportedVersion(mockk())
        val updateAvailable = InAppNotification.UpdateAvailable(mockk())
        versionNotifications.value = listOf(unsupportedVersion, updateAvailable)

        val accountExpiry = InAppNotification.AccountExpiry(DateTime.now())
        accountExpiryNotifications.value = listOf(accountExpiry)

        inAppNotificationController.notifications.test {
            val notifications = awaitItem()

            assertEquals(
                listOf(
                    tunnelStateError,
                    tunnelStateBlocked,
                    unsupportedVersion,
                    accountExpiry,
                    newDevice,
                    updateAvailable,
                ),
                notifications
            )
        }
    }
}
