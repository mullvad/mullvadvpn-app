package net.mullvad.mullvadvpn.feature.home.impl.welcome

import app.cash.turbine.test
import arrow.core.right
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.unmockkAll
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.lib.model.AccountData
import net.mullvad.mullvadvpn.lib.model.AccountNumber
import net.mullvad.mullvadvpn.lib.model.Device
import net.mullvad.mullvadvpn.lib.model.DeviceState
import net.mullvad.mullvadvpn.lib.model.DisconnectReason
import net.mullvad.mullvadvpn.lib.model.TunnelState
import net.mullvad.mullvadvpn.lib.repository.AccountRepository
import net.mullvad.mullvadvpn.lib.repository.ConnectionProxy
import net.mullvad.mullvadvpn.lib.repository.DeviceRepository
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class WelcomeViewModelTest {

    private val deviceStateFlow = MutableStateFlow<DeviceState?>(DeviceState.LoggedOut)
    private val accountExpiryStateFlow = MutableStateFlow<AccountData?>(null)

    // ConnectionProxy
    private val mockConnectionProxy: ConnectionProxy = mockk()

    // Event notifiers
    private val tunnelState = MutableStateFlow<TunnelState>(TunnelState.Disconnected())

    private val mockAccountRepository: AccountRepository = mockk(relaxed = true)
    private val mockDeviceRepository: DeviceRepository = mockk(relaxed = true)

    private lateinit var viewModel: WelcomeViewModel

    @BeforeEach
    fun setup() {
        every { mockDeviceRepository.deviceState } returns deviceStateFlow

        every { mockConnectionProxy.tunnelState } returns tunnelState

        every { mockAccountRepository.accountData } returns accountExpiryStateFlow

        viewModel =
            WelcomeViewModel(
                accountRepository = mockAccountRepository,
                deviceRepository = mockDeviceRepository,
                connectionProxy = mockConnectionProxy,
            )
    }

    @AfterEach
    fun tearDown() {
        unmockkAll()
    }

    @Test
    fun `on new TunnelState uiState should include new TunnelState`() = runTest {
        // Arrange
        val tunnelUiStateTestItem: TunnelState = mockk()

        // Act, Assert
        viewModel.uiState.test {
            // Default state
            awaitItem()
            tunnelState.emit(tunnelUiStateTestItem)
            val result = awaitItem()
            assertIs<Lc.Content<WelcomeUiState>>(result)
            assertEquals(tunnelUiStateTestItem, result.value.tunnelState)
        }
    }

    @Test
    fun `when DeviceRepository returns LoggedIn uiState should include new accountNumber`() =
        runTest {
            // Arrange
            val expectedAccountNumber = AccountNumber("4444555566667777")
            val device: Device = mockk()
            every { device.displayName() } returns ""

            // Act, Assert
            viewModel.uiState.test {
                // Default state
                awaitItem()
                deviceStateFlow.value =
                    DeviceState.LoggedIn(accountNumber = expectedAccountNumber, device = device)
                val result = awaitItem()
                assertIs<Lc.Content<WelcomeUiState>>(result)
                assertEquals(expectedAccountNumber, result.value.accountNumber)
            }
        }

    @Test
    fun `when on disconnect click is called should call connection proxy disconnect`() = runTest {
        // Arrange
        val mockDisconnectReason = DisconnectReason.USER_INITIATED_WELCOME
        coEvery { mockConnectionProxy.disconnect(any()) } returns true.right()

        // Act
        viewModel.onDisconnectClick()

        // Assert
        coVerify { mockConnectionProxy.disconnect(mockDisconnectReason) }
    }
}
