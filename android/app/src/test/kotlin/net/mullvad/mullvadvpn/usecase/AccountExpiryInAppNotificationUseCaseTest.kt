@file:OptIn(kotlinx.coroutines.ExperimentalCoroutinesApi::class)

package net.mullvad.mullvadvpn.usecase

import app.cash.turbine.test
import io.mockk.MockKAnnotations
import io.mockk.every
import io.mockk.mockk
import io.mockk.unmockkAll
import kotlin.test.assertEquals
import kotlin.test.assertTrue
import kotlin.time.Duration.Companion.seconds
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.advanceTimeBy
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.lib.model.AccountData
import net.mullvad.mullvadvpn.lib.shared.AccountRepository
import net.mullvad.mullvadvpn.repository.InAppNotification
import net.mullvad.mullvadvpn.service.notifications.accountexpiry.ACCOUNT_EXPIRY_CLOSE_TO_EXPIRY_THRESHOLD
import net.mullvad.mullvadvpn.service.notifications.accountexpiry.ACCOUNT_EXPIRY_IN_APP_NOTIFICATION_UPDATE_INTERVAL
import org.joda.time.DateTime
import org.joda.time.Duration
import org.joda.time.Period
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class AccountExpiryInAppNotificationUseCaseTest {

    private val accountExpiry = MutableStateFlow<AccountData?>(null)
    private lateinit var accountExpiryInAppNotificationUseCase:
        AccountExpiryInAppNotificationUseCase

    private lateinit var notificationThreshold: DateTime

    @BeforeEach
    fun setup() {
        MockKAnnotations.init(this)

        val accountRepository = mockk<AccountRepository>()
        every { accountRepository.accountData } returns accountExpiry

        accountExpiryInAppNotificationUseCase =
            AccountExpiryInAppNotificationUseCase(accountRepository)

        notificationThreshold = DateTime.now().plus(ACCOUNT_EXPIRY_CLOSE_TO_EXPIRY_THRESHOLD)
    }

    @AfterEach
    fun teardown() {
        unmockkAll()
    }

    @Test
    fun `initial state should be empty`() = runTest {
        accountExpiryInAppNotificationUseCase().test { assertTrue { awaitItem().isEmpty() } }
    }

    @Test
    fun `account that expires within the threshold should emit a notification`() = runTest {
        accountExpiryInAppNotificationUseCase().test {
            assertTrue { awaitItem().isEmpty() }
            val expiry = setExpiry(notificationThreshold.minusHours(1))
            assertExpiryNotificationAndPeriod(expiry, expectMostRecentItem())
            expectNoEvents()
        }
    }

    @Test
    fun `account that expires after the threshold should not emit a notification`() = runTest {
        accountExpiryInAppNotificationUseCase().test {
            assertTrue { awaitItem().isEmpty() }
            setExpiry(notificationThreshold.plusDays(1))
            expectNoEvents()
        }
    }

    @Test
    fun `should emit when the threshold is passed`() = runTest {
        accountExpiryInAppNotificationUseCase().test {
            assertTrue { awaitItem().isEmpty() }
            val expiry = setExpiry(notificationThreshold.plusMinutes(1))
            expectNoEvents()

            // Advance to before threshold
            advanceTimeBy(59.seconds)
            expectNoEvents()

            // Advance to after threshold
            advanceTimeBy(2.seconds)
            assertExpiryNotificationAndPeriod(expiry, expectMostRecentItem())
            expectNoEvents()
        }
    }

    @Test
    fun `should emit zero period when the time expires`() = runTest {
        accountExpiryInAppNotificationUseCase().test {
            assertTrue { awaitItem().isEmpty() }

            // Set expiry to to be in the final update period.
            val inLastUpdate =
                DateTime.now()
                    .plus(ACCOUNT_EXPIRY_IN_APP_NOTIFICATION_UPDATE_INTERVAL)
                    .minusSeconds(1)
            val expiry = setExpiry(inLastUpdate)

            // The expiry time is within the notification threshold so we should have an item
            // immediately.
            assertExpiryNotificationAndPeriod(expiry, expectMostRecentItem())
            expectNoEvents()

            // Advance past the delay before the while loop:
            advanceTimeBy(ACCOUNT_EXPIRY_IN_APP_NOTIFICATION_UPDATE_INTERVAL.millis)
            // Advance past the delay after the while loop:
            advanceTimeBy(ACCOUNT_EXPIRY_IN_APP_NOTIFICATION_UPDATE_INTERVAL.millis)
            assertEquals(Duration.ZERO, getExpiryNotificationDuration(expectMostRecentItem()))
            expectNoEvents()

            // Make sure we reset the list of notifications emitted when new time is added
            setExpiry(DateTime.now().plus(ACCOUNT_EXPIRY_CLOSE_TO_EXPIRY_THRESHOLD).plusDays(1))
            assertEquals(emptyList(), expectMostRecentItem())
        }
    }

    private fun setExpiry(expiryDateTime: DateTime): DateTime {
        val expiry = AccountData(mockk(relaxed = true), expiryDateTime)
        accountExpiry.value = expiry
        return expiryDateTime
    }

    // Assert that we go a single AccountExpiry notification and that the period is within
    // the expected range (checking exact period values is not possible since we use DateTime.now)
    private fun assertExpiryNotificationAndPeriod(
        expiry: DateTime,
        notifications: List<InAppNotification>,
    ) {
        val notificationDuration = getExpiryNotificationDuration(notifications)
        val periodNow = Period(DateTime.now(), expiry)
        assertTrue(periodNow.toStandardDuration() <= notificationDuration)
        assertTrue(
            periodNow.toStandardDuration().plus(Duration.standardSeconds(5)) > notificationDuration
        )
    }

    private fun getExpiryNotificationDuration(notifications: List<InAppNotification>): Duration {
        assertTrue(notifications.size == 1, "Expected a single notification")
        val n = notifications[0]
        if (n !is InAppNotification.AccountExpiry) {
            error("Expected an AccountExpiry notification")
        }
        return n.expiry
    }
}
