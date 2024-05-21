package net.mullvad.mullvadvpn.viewmodel

import app.cash.turbine.test
import io.mockk.MockKAnnotations
import io.mockk.Runs
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.coVerifyOrder
import io.mockk.every
import io.mockk.impl.annotations.MockK
import io.mockk.just
import io.mockk.mockk
import io.mockk.unmockkAll
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.compose.state.DeviceRevokedUiState
import net.mullvad.mullvadvpn.lib.account.AccountRepository
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.ui.serviceconnection.ConnectionProxy
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class DeviceRevokedViewModelTest {

    @MockK private lateinit var mockedAccountRepository: AccountRepository

    @MockK private lateinit var mockConnectionProxy: ConnectionProxy

    private lateinit var viewModel: DeviceRevokedViewModel

    private val tunnelStateFlow = MutableSharedFlow<TunnelState>()

    @BeforeEach
    fun setup() {
        MockKAnnotations.init(this)
        every { mockConnectionProxy.tunnelState } returns tunnelStateFlow
        viewModel =
            DeviceRevokedViewModel(
                accountRepository = mockedAccountRepository,
                connectionProxy = mockConnectionProxy,
                dispatcher = UnconfinedTestDispatcher()
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
    fun `onGoToLoginClicked should invoke logout on AccountRepository`() {
        // Arrange
        coEvery { mockConnectionProxy.disconnect() } just Runs
        coEvery { mockedAccountRepository.logout() } just Runs

        // Act
        viewModel.onGoToLoginClicked()

        // Assert
        coVerify { mockedAccountRepository.logout() }
    }

    @Test
    fun `onGoToLoginClicked should invoke disconnect before logout when connected`() {
        // Arrange
        coEvery { mockConnectionProxy.disconnect() } just Runs
        coEvery { mockedAccountRepository.logout() } just Runs

        // Act
        viewModel.onGoToLoginClicked()

        // Assert
        coVerifyOrder {
            mockConnectionProxy.disconnect()
            mockedAccountRepository.logout()
        }
    }
}
