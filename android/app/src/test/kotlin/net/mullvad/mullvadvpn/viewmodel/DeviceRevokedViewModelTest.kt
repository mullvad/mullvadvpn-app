package net.mullvad.mullvadvpn.viewmodel

import app.cash.turbine.test
import arrow.core.right
import io.mockk.MockKAnnotations
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.coVerifyOrder
import io.mockk.every
import io.mockk.impl.annotations.MockK
import io.mockk.mockk
import io.mockk.unmockkAll
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.compose.state.DeviceRevokedUiState
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.lib.model.TunnelState
import net.mullvad.mullvadvpn.lib.shared.AccountRepository
import net.mullvad.mullvadvpn.lib.shared.ConnectionProxy
import net.mullvad.mullvadvpn.service.notifications.accountexpiry.AccountExpiryNotificationProvider
import net.mullvad.mullvadvpn.usecase.ScheduleNotificationAlarmUseCase
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExperimentalCoroutinesApi
@ExtendWith(TestCoroutineRule::class)
class DeviceRevokedViewModelTest {

    @MockK private lateinit var mockedAccountRepository: AccountRepository

    @MockK private lateinit var mockConnectionProxy: ConnectionProxy

    private lateinit var viewModel: DeviceRevokedViewModel

    private val tunnelStateFlow = MutableSharedFlow<TunnelState>()

    private val mockScheduleNotificationAlarmUseCase =
        mockk<ScheduleNotificationAlarmUseCase>(relaxed = true)

    private val mockAccountExpiryNotificationProvider =
        mockk<AccountExpiryNotificationProvider>(relaxed = true)

    @BeforeEach
    fun setup() {
        MockKAnnotations.init(this)
        every { mockConnectionProxy.tunnelState } returns tunnelStateFlow
        viewModel =
            DeviceRevokedViewModel(
                accountRepository = mockedAccountRepository,
                connectionProxy = mockConnectionProxy,
                scheduleNotificationAlarmUseCase = mockScheduleNotificationAlarmUseCase,
                accountExpiryNotificationProvider = mockAccountExpiryNotificationProvider,
            )
    }

    @AfterEach
    fun teardown() {
        unmockkAll()
    }

    @Test
    fun `when tunnel state is secured uiState should be SECURED`() = runTest {
        // Arrange
        val tunnelState: TunnelState = mockk()
        every { tunnelState.isSecured() } returns true

        // Act, Assert
        viewModel.uiState.test {
            assertEquals(DeviceRevokedUiState.UNKNOWN, awaitItem())
            tunnelStateFlow.emit(tunnelState)
            assertEquals(DeviceRevokedUiState.SECURED, awaitItem())
        }
    }

    @Test
    fun `when subscription starts the user account expiry notification should be cancelled`() =
        runTest {
            // Act, Assert
            viewModel.uiState.test {
                assertEquals(DeviceRevokedUiState.UNKNOWN, awaitItem())
                coVerify { mockScheduleNotificationAlarmUseCase(null, null) }
                coVerify { mockAccountExpiryNotificationProvider.cancelNotification() }
            }
        }

    @Test
    fun `onGoToLoginClicked should invoke logout on AccountRepository`() {
        // Arrange
        coEvery { mockConnectionProxy.disconnect() } returns true.right()
        coEvery { mockedAccountRepository.logout() } returns Unit.right()

        // Act
        viewModel.onGoToLoginClicked()

        // Assert
        coVerify { mockedAccountRepository.logout() }
    }

    @Test
    fun `onGoToLoginClicked should invoke disconnect before logout when connected`() {
        // Arrange
        coEvery { mockConnectionProxy.disconnect() } returns true.right()
        coEvery { mockedAccountRepository.logout() } returns Unit.right()

        // Act
        viewModel.onGoToLoginClicked()

        // Assert
        coVerifyOrder {
            mockConnectionProxy.disconnect()
            mockedAccountRepository.logout()
        }
    }
}
