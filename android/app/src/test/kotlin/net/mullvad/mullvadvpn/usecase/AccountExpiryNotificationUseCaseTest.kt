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
import net.mullvad.mullvadvpn.repository.InAppNotification
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionAccountDataSource
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionContainer
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionState
import org.joda.time.DateTime
import org.junit.After
import org.junit.Before
import org.junit.Rule
import org.junit.Test

class AccountExpiryNotificationUseCaseTest {
    @get:Rule val testCoroutineRule = TestCoroutineRule()

    private val mockServiceConnectionManager: ServiceConnectionManager = mockk()
    private val mockServiceConnectionContainer: ServiceConnectionContainer = mockk()
    private val serviceConnectionState =
        MutableStateFlow<ServiceConnectionState>(ServiceConnectionState.Disconnected)
    private val accountExpiry = MutableStateFlow<AccountExpiry>(AccountExpiry.Missing)
    private lateinit var accountExpiryNotificationUseCase: AccountExpiryNotificationUseCase

    @Before
    fun setup() {
        MockKAnnotations.init(this)

        val accountDataSource = mockk<ServiceConnectionAccountDataSource>()
        every { mockServiceConnectionManager.connectionState } returns serviceConnectionState
        every { mockServiceConnectionContainer.accountDataSource } returns accountDataSource
        every { accountDataSource.accountExpiry } returns accountExpiry

        accountExpiryNotificationUseCase =
            AccountExpiryNotificationUseCase(mockServiceConnectionManager)
    }

    @After
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
            val expiryDate = DateTime.now().plusDays(2)
            accountExpiry.value = AccountExpiry.Available(expiryDate)
            serviceConnectionState.value =
                ServiceConnectionState.ConnectedReady(mockServiceConnectionContainer)
            assertEquals(listOf(InAppNotification.AccountExpiry(expiryDate)), awaitItem())
        }
    }

    @Test
    fun `ensure an expire of 4 days in the future does not produce a notification`() = runTest {
        // Arrange, Act, Assert
        accountExpiryNotificationUseCase.notifications().test {
            assertTrue { awaitItem().isEmpty() }
            accountExpiry.value = AccountExpiry.Available(DateTime.now().plusDays(4))
            serviceConnectionState.value =
                ServiceConnectionState.ConnectedReady(mockServiceConnectionContainer)
            expectNoEvents()
        }
    }
}
