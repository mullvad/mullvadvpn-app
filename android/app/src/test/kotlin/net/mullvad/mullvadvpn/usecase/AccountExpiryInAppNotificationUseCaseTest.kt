package net.mullvad.mullvadvpn.usecase

import app.cash.turbine.test
import io.mockk.MockKAnnotations
import io.mockk.every
import io.mockk.mockk
import io.mockk.unmockkAll
import kotlin.math.roundToInt
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
        // Arrange, Act, Assert
        accountExpiryInAppNotificationUseCase().test { assertTrue { awaitItem().isEmpty() } }
    }

    @Test
    fun `account that expires within the threshold should emit a notification`() = runTest {
        // Arrange, Act, Assert
        accountExpiryInAppNotificationUseCase().test {
            assertTrue { awaitItem().isEmpty() }
            val expiry = setExpiry(notificationThreshold.minusHours(1))
            assertExpiryNotificationAndPeriod(expiry, expectMostRecentItem())
            expectNoEvents()
        }
    }

    @Test
    fun `account that expires after the threshold should not emit a notification`() = runTest {
        // Arrange, Act, Assert
        accountExpiryInAppNotificationUseCase().test {
            assertTrue { awaitItem().isEmpty() }
            setExpiry(notificationThreshold.plusDays(1))
            expectNoEvents()
        }
    }

    @Test
    fun `should emit when the threshold is passed`() = runTest {
        // Arrange, Act, Assert
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
    fun `should emit remaining time divided by update interval time times`() = runTest {
        // Arrange, Act, Assert
        accountExpiryInAppNotificationUseCase().test {
            assertTrue { awaitItem().isEmpty() }

            // Set expiry to to be 1 second before the end of the first update interval
            val beforeUpdate =
                notificationThreshold
                    .minus(ACCOUNT_EXPIRY_IN_APP_NOTIFICATION_UPDATE_INTERVAL)
                    .plusSeconds(1)
            val expiry = setExpiry(beforeUpdate)

            // The expiry time is within the notification threshold so we should have an item
            // immediately.
            assertExpiryNotificationAndPeriod(expiry, expectMostRecentItem())
            expectNoEvents()

            val expectedEmits =
                (Duration(DateTime.now(), beforeUpdate).millis.toDouble() /
                        ACCOUNT_EXPIRY_IN_APP_NOTIFICATION_UPDATE_INTERVAL.millis)
                    .roundToInt()

            advanceTimeBy(2.seconds)
            repeat(expectedEmits) {
                expectMostRecentItem()
                advanceTimeBy(
                    ACCOUNT_EXPIRY_IN_APP_NOTIFICATION_UPDATE_INTERVAL.plus(
                            Duration.standardSeconds(1)
                        )
                        .millis
                )
            }

            expectNoEvents()
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
        assertTrue(notifications.size == 1, "Expected a single notification")
        val n = notifications[0]
        if (n !is InAppNotification.AccountExpiry) {
            error("Expected an AccountExpiry notification")
        }
        val periodNow = Period(DateTime.now(), expiry)
        assertTrue(periodNow.toStandardDuration() <= n.expiry.toStandardDuration())
        assertTrue(
            periodNow.toStandardDuration().plus(Duration.standardSeconds(5)) >
                n.expiry.toStandardDuration()
        )
    }
}
