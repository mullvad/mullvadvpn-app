package net.mullvad.mullvadvpn

import app.cash.turbine.test
import io.mockk.MockKAnnotations
import io.mockk.every
import io.mockk.mockk
import io.mockk.unmockkAll
import java.time.Duration
import kotlin.test.assertEquals
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.lib.model.ErrorState
import net.mullvad.mullvadvpn.repository.InAppNotification
import net.mullvad.mullvadvpn.repository.InAppNotificationController
import net.mullvad.mullvadvpn.usecase.AccountExpiryInAppNotificationUseCase
import net.mullvad.mullvadvpn.usecase.NewChangelogNotificationUseCase
import net.mullvad.mullvadvpn.usecase.NewDeviceNotificationUseCase
import net.mullvad.mullvadvpn.usecase.TunnelStateNotificationUseCase
import net.mullvad.mullvadvpn.usecase.VersionNotificationUseCase
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExperimentalCoroutinesApi
@ExtendWith(TestCoroutineRule::class)
class InAppNotificationControllerTest {

    private lateinit var inAppNotificationController: InAppNotificationController
    private val accountExpiryNotifications = MutableStateFlow(emptyList<InAppNotification>())
    private val newDeviceNotifications = MutableStateFlow(emptyList<InAppNotification.NewDevice>())
    private val newVersionChangelogNotifications =
        MutableStateFlow(emptyList<InAppNotification.NewVersionChangelog>())
    private val versionNotifications = MutableStateFlow(emptyList<InAppNotification>())
    private val tunnelStateNotifications = MutableStateFlow(emptyList<InAppNotification>())

    private lateinit var job: Job

    @BeforeEach
    fun setup() {
        MockKAnnotations.init(this)

        val accountExpiryInAppNotificationUseCase: AccountExpiryInAppNotificationUseCase = mockk()
        val newDeviceNotificationUseCase: NewDeviceNotificationUseCase = mockk()
        val newVersionChangelogUseCase: NewChangelogNotificationUseCase = mockk()
        val versionNotificationUseCase: VersionNotificationUseCase = mockk()
        val tunnelStateNotificationUseCase: TunnelStateNotificationUseCase = mockk()
        every { accountExpiryInAppNotificationUseCase.invoke() } returns accountExpiryNotifications
        every { newDeviceNotificationUseCase.invoke() } returns newDeviceNotifications
        every { newVersionChangelogUseCase.invoke() } returns newVersionChangelogNotifications
        every { versionNotificationUseCase.invoke() } returns versionNotifications
        every { versionNotificationUseCase.invoke() } returns versionNotifications
        every { tunnelStateNotificationUseCase.invoke() } returns tunnelStateNotifications
        job = Job()

        inAppNotificationController =
            InAppNotificationController(
                accountExpiryInAppNotificationUseCase,
                newDeviceNotificationUseCase,
                newVersionChangelogUseCase,
                versionNotificationUseCase,
                tunnelStateNotificationUseCase,
                CoroutineScope(job + UnconfinedTestDispatcher()),
            )
    }

    @AfterEach
    fun teardown() {
        job.cancel()
        unmockkAll()
    }

    @Test
    fun `ensure all notifications have the right priority`() = runTest {
        val newDevice = InAppNotification.NewDevice("")
        newDeviceNotifications.value = listOf(newDevice)

        val newVersionChangelog = InAppNotification.NewVersionChangelog
        newVersionChangelogNotifications.value = listOf(newVersionChangelog)

        val errorState: ErrorState = mockk()
        val tunnelStateBlocked = InAppNotification.TunnelStateBlocked
        val tunnelStateError = InAppNotification.TunnelStateError(errorState)
        tunnelStateNotifications.value = listOf(tunnelStateBlocked, tunnelStateError)

        val unsupportedVersion = InAppNotification.UnsupportedVersion(mockk())
        versionNotifications.value = listOf(unsupportedVersion)

        val accountExpiry = InAppNotification.AccountExpiry(Duration.ZERO)
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
                    newVersionChangelog,
                ),
                notifications,
            )
        }
    }
}
