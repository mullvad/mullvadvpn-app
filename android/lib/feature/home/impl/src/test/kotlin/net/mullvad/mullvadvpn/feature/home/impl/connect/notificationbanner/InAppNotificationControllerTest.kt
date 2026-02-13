package net.mullvad.mullvadvpn.feature.home.impl.connect.notificationbanner

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
import net.mullvad.mullvadvpn.lib.model.InAppNotification
import net.mullvad.mullvadvpn.lib.usecase.inappnotification.AccountExpiryInAppNotificationUseCase
import net.mullvad.mullvadvpn.lib.usecase.inappnotification.NewChangelogNotificationUseCase
import net.mullvad.mullvadvpn.lib.usecase.inappnotification.NewDeviceNotificationUseCase
import net.mullvad.mullvadvpn.lib.usecase.inappnotification.TunnelStateNotificationUseCase
import net.mullvad.mullvadvpn.lib.usecase.inappnotification.VersionNotificationUseCase
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExperimentalCoroutinesApi
@ExtendWith(TestCoroutineRule::class)
class InAppNotificationControllerTest {

    private lateinit var inAppNotificationController: InAppNotificationController
    private val accountExpiryNotifications =
        MutableStateFlow<InAppNotification.AccountExpiry?>(null)
    private val newDeviceNotifications = MutableStateFlow<InAppNotification.NewDevice?>(null)
    private val newVersionChangelogNotifications =
        MutableStateFlow<InAppNotification.NewVersionChangelog?>(null)
    private val versionNotifications = MutableStateFlow<InAppNotification.UnsupportedVersion?>(null)
    private val tunnelStateNotifications = MutableStateFlow<InAppNotification?>(null)

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
                listOf(
                    accountExpiryInAppNotificationUseCase,
                    newDeviceNotificationUseCase,
                    newVersionChangelogUseCase,
                    versionNotificationUseCase,
                    tunnelStateNotificationUseCase,
                ),
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
        newDeviceNotifications.value = newDevice

        val newVersionChangelog = InAppNotification.NewVersionChangelog
        newVersionChangelogNotifications.value = newVersionChangelog

        val errorState: ErrorState = mockk()
        every { errorState.cause } returns mockk()
        val tunnelStateBlocked = InAppNotification.TunnelStateBlocked
        tunnelStateNotifications.value = tunnelStateBlocked

        val unsupportedVersion = InAppNotification.UnsupportedVersion(mockk())
        versionNotifications.value = unsupportedVersion

        val accountExpiry = InAppNotification.AccountExpiry(Duration.ZERO)
        accountExpiryNotifications.value = accountExpiry

        inAppNotificationController.notifications.test {
            val notifications = awaitItem()

            assertEquals(
                listOf(
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
