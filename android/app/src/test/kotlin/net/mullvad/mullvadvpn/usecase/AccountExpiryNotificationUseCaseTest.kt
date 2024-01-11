package net.mullvad.mullvadvpn.usecase

import app.cash.turbine.test
import io.mockk.MockKAnnotations
import io.mockk.every
import io.mockk.mockk
import io.mockk.unmockkAll
import kotlin.test.assertEquals
import kotlin.test.assertTrue
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.model.AccountExpiry
import net.mullvad.mullvadvpn.repository.AccountRepository
import net.mullvad.mullvadvpn.repository.InAppNotification
import org.joda.time.DateTime
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class AccountExpiryNotificationUseCaseTest {

    private val accountExpiry = MutableStateFlow<AccountExpiry>(AccountExpiry.Missing)
    private lateinit var accountExpiryNotificationUseCase: AccountExpiryNotificationUseCase

    @BeforeEach
    fun setup() {
        MockKAnnotations.init(this)

        val accountRepository = mockk<AccountRepository>()
        every { accountRepository.accountExpiryState } returns accountExpiry

        accountExpiryNotificationUseCase = AccountExpiryNotificationUseCase(accountRepository)
    }

    @AfterEach
    fun teardown() {
        unmockkAll()
    }

    @Test
    fun `ensure notifications are empty by default`() = runTest {
        // Arrange, Act, Assert
        accountExpiryNotificationUseCase.notifications().test {
            assertTrue { awaitItem().isEmpty() }
        }
    }

    @Test
    fun `ensure account expiry within 3 days generates notification`() = runTest {
        // Arrange, Act, Assert
        accountExpiryNotificationUseCase.notifications().test {
            assertTrue { awaitItem().isEmpty() }
            val closeToExpiry = AccountExpiry.Available(DateTime.now().plusDays(2))
            accountExpiry.value = closeToExpiry

            assertEquals(
                listOf(InAppNotification.AccountExpiry(closeToExpiry.expiryDateTime)),
                awaitItem()
            )
        }
    }

    @Test
    fun `ensure an expire of 4 days in the future does not produce a notification`() = runTest {
        // Arrange, Act, Assert
        accountExpiryNotificationUseCase.notifications().test {
            assertTrue { awaitItem().isEmpty() }
            accountExpiry.value = AccountExpiry.Available(DateTime.now().plusDays(4))
            expectNoEvents()
        }
    }
}
