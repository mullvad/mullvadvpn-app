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
import net.mullvad.mullvadvpn.lib.account.AccountRepository
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.model.AccountData
import net.mullvad.mullvadvpn.repository.InAppNotification
import org.joda.time.DateTime
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class AccountExpiryNotificationUseCaseTest {

    private val accountExpiry = MutableStateFlow<AccountData>(AccountData.Missing)
    private lateinit var accountExpiryNotificationUseCase: AccountExpiryNotificationUseCase

    @BeforeEach
    fun setup() {
        MockKAnnotations.init(this)

        val accountRepository = mockk<net.mullvad.mullvadvpn.lib.account.AccountRepository>()
        every { accountRepository.accountExpiryState } returns accountExpiry

        accountExpiryNotificationUseCase = AccountExpiryNotificationUseCase(accountRepository)
    }

    @AfterEach
    fun teardown() {
        unmockkAll()
    }

    @Test
    fun `initial state should be empty`() = runTest {
        // Arrange, Act, Assert
        accountExpiryNotificationUseCase.notifications().test {
            assertTrue { awaitItem().isEmpty() }
        }
    }

    @Test
    fun `account that expires within 3 days should emit a notification`() = runTest {
        // Arrange, Act, Assert
        accountExpiryNotificationUseCase.notifications().test {
            assertTrue { awaitItem().isEmpty() }
            val closeToExpiry = AccountData.Available(DateTime.now().plusDays(2))
            accountExpiry.value = closeToExpiry

            assertEquals(
                listOf(InAppNotification.AccountExpiry(closeToExpiry.expiryDateTime)),
                awaitItem()
            )
        }
    }

    @Test
    fun `account that expires in 4 days should not emit a notification`() = runTest {
        // Arrange, Act, Assert
        accountExpiryNotificationUseCase.notifications().test {
            assertTrue { awaitItem().isEmpty() }
            accountExpiry.value = AccountData.Available(DateTime.now().plusDays(4))
            expectNoEvents()
        }
    }
}
