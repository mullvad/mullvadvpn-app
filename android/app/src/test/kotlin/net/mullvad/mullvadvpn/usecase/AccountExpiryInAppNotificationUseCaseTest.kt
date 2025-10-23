@file:OptIn(kotlinx.coroutines.ExperimentalCoroutinesApi::class)

package net.mullvad.mullvadvpn.usecase

import app.cash.turbine.test
import io.mockk.MockKAnnotations
import io.mockk.every
import io.mockk.mockk
import io.mockk.unmockkAll
import java.time.Duration
import java.time.ZonedDateTime
import kotlin.test.assertEquals
import kotlin.test.assertTrue
import kotlin.time.Duration.Companion.seconds
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.advanceTimeBy
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.data.mock
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.lib.model.AccountData
import net.mullvad.mullvadvpn.lib.model.InAppNotification
import net.mullvad.mullvadvpn.lib.shared.AccountRepository
import net.mullvad.mullvadvpn.service.notifications.accountexpiry.ACCOUNT_EXPIRY_CLOSE_TO_EXPIRY_THRESHOLD
import net.mullvad.mullvadvpn.service.notifications.accountexpiry.ACCOUNT_EXPIRY_NOTIFICATION_UPDATE_INTERVAL
import net.mullvad.mullvadvpn.usecase.inappnotification.AccountExpiryInAppNotificationUseCase
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.assertNull
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class AccountExpiryInAppNotificationUseCaseTest {

    private val accountExpiry = MutableStateFlow<AccountData?>(null)
    private lateinit var accountExpiryInAppNotificationUseCase:
        AccountExpiryInAppNotificationUseCase

    private lateinit var notificationThreshold: ZonedDateTime

    @BeforeEach
    fun setup() {
        MockKAnnotations.init(this)

        val accountRepository = mockk<AccountRepository>()
        every { accountRepository.accountData } returns accountExpiry

        accountExpiryInAppNotificationUseCase =
            AccountExpiryInAppNotificationUseCase(accountRepository)

        notificationThreshold = ZonedDateTime.now().plus(ACCOUNT_EXPIRY_CLOSE_TO_EXPIRY_THRESHOLD)
    }

    @AfterEach
    fun teardown() {
        unmockkAll()
    }

    @Test
    fun `initial state should be null`() = runTest {
        accountExpiryInAppNotificationUseCase().test { assertNull(awaitItem()) }
    }

    @Test
    fun `account that expires within the threshold should emit a notification`() = runTest {
        accountExpiryInAppNotificationUseCase().test {
            assertNull(awaitItem())
            val expiry = setExpiry(notificationThreshold.minusHours(1))
            assertExpiryNotificationDuration(expiry, expectMostRecentItem())
            expectNoEvents()
        }
    }

    @Test
    fun `account that expires after the threshold should not emit a notification`() = runTest {
        accountExpiryInAppNotificationUseCase().test {
            assertNull(awaitItem())
            setExpiry(notificationThreshold.plusHours(24))
            expectNoEvents()
        }
    }

    @Test
    fun `should emit when the threshold is passed`() = runTest {
        accountExpiryInAppNotificationUseCase().test {
            assertNull(awaitItem())
            val expiry = setExpiry(notificationThreshold.plusMinutes(1))
            expectNoEvents()

            // Advance to before threshold
            advanceTimeBy(59.seconds)
            expectNoEvents()

            // Advance to after threshold
            advanceTimeBy(2.seconds)
            assertExpiryNotificationDuration(expiry, expectMostRecentItem())
            expectNoEvents()
        }
    }

    @Test
    fun `should emit zero duration when the time expires`() = runTest {
        accountExpiryInAppNotificationUseCase().test {
            assertNull(awaitItem())

            // Set expiry to to be in the final update interval.
            val inLastUpdate =
                ZonedDateTime.now()
                    .plus(ACCOUNT_EXPIRY_NOTIFICATION_UPDATE_INTERVAL)
                    .minusSeconds(1)
            val expiry = setExpiry(inLastUpdate)

            // The expiry time is within the notification threshold so we should have an item
            // immediately.
            assertExpiryNotificationDuration(expiry, expectMostRecentItem())
            expectNoEvents()

            // Advance past the delay before the while loop:
            advanceTimeBy(ACCOUNT_EXPIRY_NOTIFICATION_UPDATE_INTERVAL.toMillis())
            // Advance past the delay after the while loop:
            advanceTimeBy(ACCOUNT_EXPIRY_NOTIFICATION_UPDATE_INTERVAL.toMillis())
            assertEquals(Duration.ZERO, getExpiryNotificationDuration(expectMostRecentItem()))
            expectNoEvents()

            // Make sure we reset the list of notifications emitted when new time is added
            setExpiry(
                ZonedDateTime.now().plus(ACCOUNT_EXPIRY_CLOSE_TO_EXPIRY_THRESHOLD).plusHours(24)
            )
            assertNull(expectMostRecentItem())
        }
    }

    private fun setExpiry(expiryDateTime: ZonedDateTime): ZonedDateTime {
        val expiry = AccountData.mock(expiryDateTime)
        accountExpiry.value = expiry
        return expiryDateTime
    }

    // Assert that we got a single AccountExpiry notification and that the expiry duration is within
    // the expected range (checking exact duration value is not possible since we use
    // ZonedDateTime.now)
    private fun assertExpiryNotificationDuration(
        expiry: ZonedDateTime,
        notification: InAppNotification?,
    ) {
        val notificationDuration = getExpiryNotificationDuration(notification)
        val expiresFromNow = Duration.between(ZonedDateTime.now(), expiry)
        assertTrue(expiresFromNow <= notificationDuration)
        assertTrue(expiresFromNow.plus(Duration.ofSeconds(5)) > notificationDuration)
    }

    private fun getExpiryNotificationDuration(notification: InAppNotification?): Duration {
        if (notification !is InAppNotification.AccountExpiry) {
            error("Expected an AccountExpiry notification")
        }
        return notification.expiry
    }
}
