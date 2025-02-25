package net.mullvad.mullvadvpn

import app.cash.turbine.test
import io.mockk.MockKAnnotations
import io.mockk.every
import io.mockk.mockk
import io.mockk.unmockkAll
import java.time.Duration
import java.time.ZonedDateTime
import kotlin.test.assertEquals
import kotlin.test.assertTrue
import kotlin.time.Duration.Companion.minutes
import kotlin.time.Duration.Companion.seconds
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.advanceTimeBy
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.lib.model.AccountData
import net.mullvad.mullvadvpn.lib.model.DeviceState
import net.mullvad.mullvadvpn.lib.model.Notification
import net.mullvad.mullvadvpn.lib.model.NotificationChannelId
import net.mullvad.mullvadvpn.lib.model.NotificationUpdate
import net.mullvad.mullvadvpn.lib.model.NotificationUpdate.Cancel
import net.mullvad.mullvadvpn.lib.model.NotificationUpdate.Notify
import net.mullvad.mullvadvpn.lib.shared.AccountRepository
import net.mullvad.mullvadvpn.lib.shared.DeviceRepository
import net.mullvad.mullvadvpn.service.notifications.accountexpiry.ACCOUNT_EXPIRY_CLOSE_TO_EXPIRY_THRESHOLD
import net.mullvad.mullvadvpn.service.notifications.accountexpiry.AccountExpiryNotificationProvider
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExperimentalCoroutinesApi
@ExtendWith(TestCoroutineRule::class)
class AccountExpiryNotificationProviderTest {

    private lateinit var provider: AccountExpiryNotificationProvider

    private val accountData = MutableStateFlow<AccountData?>(null)
    private val deviceState = MutableStateFlow<DeviceState?>(null)
    private val isNewDevice = MutableStateFlow(true)

    @BeforeEach
    fun setup() {
        MockKAnnotations.init(this)

        val accountRepository = mockk<AccountRepository>(relaxed = true)
        every { accountRepository.accountData } returns accountData
        every { accountRepository.isNewAccount } returns isNewDevice

        val deviceRepository = mockk<DeviceRepository>(relaxed = true)
        every { deviceRepository.deviceState } returns deviceState

        provider =
            AccountExpiryNotificationProvider(
                channelId = NotificationChannelId("channelId"),
                accountRepository = accountRepository,
                deviceRepository = deviceRepository,
            )

        deviceState.value = DeviceState.LoggedIn(mockk(relaxed = true), mockk(relaxed = true))
        isNewDevice.value = false
    }

    @AfterEach
    fun teardown() {
        unmockkAll()
    }

    @Test
    fun `should not emit notification in initial state`() = runTest {
        accountData.value = null
        deviceState.value = null
        isNewDevice.value = true
        provider.notifications.test { expectNoEvents() }
    }

    @Test
    fun `should emit notification if expiry time is shorter than expiry warning threshold`() =
        runTest {
            setExpiry(
                ZonedDateTime.now().plus(ACCOUNT_EXPIRY_CLOSE_TO_EXPIRY_THRESHOLD).minusDays(1)
            )
            provider.notifications.test {
                assertTrue(awaitItem() is Notify)
                expectNoEvents()
            }
        }

    @Test
    fun `should emit cancel notification if user account is new`() = runTest {
        isNewDevice.value = true
        setExpiry(ZonedDateTime.now().plus(ACCOUNT_EXPIRY_CLOSE_TO_EXPIRY_THRESHOLD).minusDays(1))
        provider.notifications.test {
            assertTrue(awaitItem() is Cancel)
            expectNoEvents()
        }
    }

    @Test
    fun `should emit cancel notification if user account is logged out`() = runTest {
        setIsLoggedIn(false)
        setExpiry(ZonedDateTime.now().plus(ACCOUNT_EXPIRY_CLOSE_TO_EXPIRY_THRESHOLD).minusDays(1))
        provider.notifications.test {
            assertTrue(awaitItem() is Cancel)
            expectNoEvents()

            setIsLoggedIn(true)
            assertTrue(awaitItem() is Notify)
            expectNoEvents()

            setIsLoggedIn(false)
            assertTrue(awaitItem() is Cancel)
            expectNoEvents()
        }
    }

    @Test
    fun `should emit zero duration notification when remaining time runs out`() = runTest {
        setExpiry(ZonedDateTime.now().plus(Duration.ofSeconds(60)))
        provider.notifications.test {
            assertTrue(awaitItem() is Notify)
            expectNoEvents()

            advanceTimeBy(59.seconds)
            expectNoEvents()

            advanceTimeBy(2.seconds)
            val item = getAccountExpiry(awaitItem())
            assertEquals(item.durationUntilExpiry, Duration.ZERO)
            expectNoEvents()
        }
    }

    @Test
    fun `should emit notification when update interval is passed`() = runTest {
        setExpiry(
            ZonedDateTime.now()
                .plus(ACCOUNT_EXPIRY_CLOSE_TO_EXPIRY_THRESHOLD)
                .minusDays(1)
                .plusHours(1)
        )
        provider.notifications.test {
            assertTrue(awaitItem() is Notify)
            expectNoEvents()

            advanceTimeBy(59.minutes)
            expectNoEvents()

            advanceTimeBy(1.minutes + 1.seconds)
            assertTrue(awaitItem() is Notify)
            expectNoEvents()
        }
    }

    @Test
    fun `should cancel existing notification if more time is added to account`() = runTest {
        setExpiry(ZonedDateTime.now().plus(ACCOUNT_EXPIRY_CLOSE_TO_EXPIRY_THRESHOLD).minusDays(1))
        provider.notifications.test {
            assertTrue(awaitItem() is Notify)
            expectNoEvents()

            setExpiry(
                ZonedDateTime.now().plus(ACCOUNT_EXPIRY_CLOSE_TO_EXPIRY_THRESHOLD).plusDays(1)
            )
            assertTrue(awaitItem() is Cancel)
            expectNoEvents()
        }
    }

    @Test
    fun `should not cancel existing notification if too little time is added`() = runTest {
        setExpiry(ZonedDateTime.now().plus(ACCOUNT_EXPIRY_CLOSE_TO_EXPIRY_THRESHOLD).minusDays(1))
        provider.notifications.test {
            assertTrue(awaitItem() is Notify)
            expectNoEvents()

            setExpiry(
                ZonedDateTime.now().plus(ACCOUNT_EXPIRY_CLOSE_TO_EXPIRY_THRESHOLD).minusHours(1)
            )
            assertTrue(awaitItem() is Notify)
            expectNoEvents()
        }
    }

    private fun getAccountExpiry(
        awaitItem: NotificationUpdate<Notification.AccountExpiry>
    ): Notification.AccountExpiry =
        when (awaitItem) {
            is Cancel -> error("expected AccountExpiry, was Cancel")
            is Notify -> awaitItem.value
        }

    private fun setExpiry(expiryDateTime: ZonedDateTime): ZonedDateTime {
        val expiry = AccountData(mockk(relaxed = true), expiryDateTime)
        accountData.value = expiry
        return expiryDateTime
    }

    private fun setIsLoggedIn(isLoggedIn: Boolean) {
        deviceState.value =
            if (isLoggedIn) {
                DeviceState.LoggedIn(
                    accountNumber = mockk(relaxed = true),
                    device = mockk(relaxed = true),
                )
            } else {
                DeviceState.LoggedOut
            }
    }
}
