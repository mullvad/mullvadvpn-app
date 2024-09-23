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
import net.mullvad.mullvadvpn.constant.ACCOUNT_EXPIRY_CLOSE_TO_EXPIRY_THRESHOLD
import net.mullvad.mullvadvpn.constant.ACCOUNT_EXPIRY_IN_APP_NOTIFICATION_UPDATE_INTERVAL
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.lib.model.AccountData
import net.mullvad.mullvadvpn.lib.shared.AccountRepository
import net.mullvad.mullvadvpn.repository.InAppNotification
import org.joda.time.DateTime
import org.joda.time.Duration
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class AccountExpiryNotificationUseCaseTest {

    private val accountExpiry = MutableStateFlow<AccountData?>(null)
    private lateinit var accountExpiryNotificationUseCase: AccountExpiryNotificationUseCase

    private lateinit var notificationThreshold: DateTime

    @BeforeEach
    fun setup() {
        MockKAnnotations.init(this)

        val accountRepository = mockk<AccountRepository>()
        every { accountRepository.accountData } returns accountExpiry

        accountExpiryNotificationUseCase = AccountExpiryNotificationUseCase(accountRepository)

        notificationThreshold = DateTime.now().plus(ACCOUNT_EXPIRY_CLOSE_TO_EXPIRY_THRESHOLD)
    }

    @AfterEach
    fun teardown() {
        unmockkAll()
    }

    @Test
    fun `initial state should be empty`() = runTest {
        // Arrange, Act, Assert
        accountExpiryNotificationUseCase().test { assertTrue { awaitItem().isEmpty() } }
    }

    @Test
    fun `account that expires within the threshold should emit a notification`() = runTest {
        // Arrange, Act, Assert
        accountExpiryNotificationUseCase().test {
            assertTrue { awaitItem().isEmpty() }
            val closeToExpiry = setExpiry(notificationThreshold.minusHours(1))

            assertEquals(
                listOf(InAppNotification.AccountExpiry(closeToExpiry)),
                expectMostRecentItem(),
            )
        }
    }

    @Test
    fun `account that expires after the threshold should not emit a notification`() = runTest {
        // Arrange, Act, Assert
        accountExpiryNotificationUseCase().test {
            assertTrue { awaitItem().isEmpty() }
            setExpiry(notificationThreshold.plusDays(1))
            expectNoEvents()
        }
    }

    @Test
    fun `should emit when the threshold is passed`() = runTest {
        // Arrange, Act, Assert
        accountExpiryNotificationUseCase().test {
            assertTrue { awaitItem().isEmpty() }
            val expiry = setExpiry(notificationThreshold.plusMinutes(1))
            expectNoEvents()

            // Advance to before threshold
            advanceTimeBy(59.seconds)
            expectNoEvents()

            // Advance to after threshold
            advanceTimeBy(2.seconds)
            assertEquals(listOf(InAppNotification.AccountExpiry(expiry)), expectMostRecentItem())
            expectNoEvents()
        }
    }

    @Test
    fun `should emit when the update duration is passed`() = runTest {
        // Arrange, Act, Assert
        accountExpiryNotificationUseCase().test {
            assertTrue { awaitItem().isEmpty() }
            val expiry = setExpiry(notificationThreshold.minusDays(1))
            // The expiry time is within the threshold so we should have an item immediately.
            assertEquals(listOf(InAppNotification.AccountExpiry(expiry)), expectMostRecentItem())
            expectNoEvents()

            // Advance to before update threshold
            advanceTimeBy(
                ACCOUNT_EXPIRY_IN_APP_NOTIFICATION_UPDATE_INTERVAL.minus(
                        Duration.standardSeconds(1)
                    )
                    .millis
            )
            expectNoEvents()

            // Advance to after update threshold
            advanceTimeBy(2.seconds)
            assertEquals(listOf(InAppNotification.AccountExpiry(expiry)), expectMostRecentItem())
            expectNoEvents()
        }
    }

    @Test
    fun `should stop emitting when less time than the update duration remains`() = runTest {
        // Arrange, Act, Assert
        accountExpiryNotificationUseCase().test {
            assertTrue { awaitItem().isEmpty() }
            // Set expiry to a time that is 1 second less than the update interval.
            val expiry =
                setExpiry(
                    DateTime.now()
                        .plus(ACCOUNT_EXPIRY_IN_APP_NOTIFICATION_UPDATE_INTERVAL)
                        .minusSeconds(1)
                )
            // The expiry time is within the notification threshold so we should have an item
            // immediately.
            assertEquals(listOf(InAppNotification.AccountExpiry(expiry)), expectMostRecentItem())
            expectNoEvents()

            // Advance to after update threshold
            advanceTimeBy(
                ACCOUNT_EXPIRY_IN_APP_NOTIFICATION_UPDATE_INTERVAL.plus(Duration.standardSeconds(1))
                    .millis
            )
            // Because the remaining time was less than the update threshold no more items should
            // have been emitted.
            expectNoEvents()
        }
    }

    private fun setExpiry(expiryDateTime: DateTime): DateTime {
        val expiry = AccountData(mockk(relaxed = true), expiryDateTime)
        accountExpiry.value = expiry
        return expiryDateTime
    }
}
