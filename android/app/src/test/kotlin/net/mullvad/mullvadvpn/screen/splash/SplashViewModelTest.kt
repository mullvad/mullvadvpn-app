package net.mullvad.mullvadvpn.screen.splash

import app.cash.turbine.test
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import java.time.ZonedDateTime
import kotlin.test.assertEquals
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.lib.model.AccountData
import net.mullvad.mullvadvpn.lib.model.AccountNumber
import net.mullvad.mullvadvpn.lib.model.DeviceState
import net.mullvad.mullvadvpn.lib.repository.AccountRepository
import net.mullvad.mullvadvpn.lib.repository.DeviceRepository
import net.mullvad.mullvadvpn.lib.repository.SplashCompleteRepository
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class SplashViewModelTest {
    private val mockAccountRepository: AccountRepository = mockk(relaxed = true)
    private val mockDeviceRepository: DeviceRepository = mockk()
    private val mockSplashCompleteRepository: SplashCompleteRepository = mockk(relaxed = true)

    private val splashViewModel: SplashViewModel =
        SplashViewModel(
            accountRepository = mockAccountRepository,
            deviceRepository = mockDeviceRepository,
            splashCompleteRepository = mockSplashCompleteRepository,
        )

    private val deviceStateFlow = MutableStateFlow<DeviceState?>(null)
    private val accountDataFlow = MutableStateFlow<AccountData?>(null)

    @BeforeEach
    fun setup() {
        every { mockDeviceRepository.deviceState } returns deviceStateFlow
        every { mockAccountRepository.accountData } returns accountDataFlow
    }

    @Test
    fun `when logged in and out of time should navigate to OutOfTime`() = runTest {
        // Arrange
        deviceStateFlow.update { DeviceState.LoggedIn(AccountNumber("11"), mockk()) }
        accountDataFlow.update {
            AccountData(
                mockk(relaxed = true),
                AccountNumber("11"),
                expiryDate = ZonedDateTime.now().minusDays(1),
            )
        }

        // Act, Assert
        splashViewModel.uiSideEffect.test {
            assertEquals(expected = SplashUiSideEffect.NavigateToOutOfTime, actual = awaitItem())
            awaitComplete()
        }
        // Make sure we have tried to update the refresh account data as well
        coVerify { mockAccountRepository.refreshAccountData() }
    }

    @Test
    fun `when logged in and have time left should navigate to Connect`() = runTest {
        // Arrange
        deviceStateFlow.update { DeviceState.LoggedIn(AccountNumber("11"), mockk()) }
        accountDataFlow.update {
            AccountData(
                mockk(relaxed = true),
                AccountNumber("11"),
                expiryDate = ZonedDateTime.now().plusDays(1),
            )
        }

        // Act, Assert
        splashViewModel.uiSideEffect.test {
            assertEquals(expected = SplashUiSideEffect.NavigateToConnect, actual = awaitItem())
            awaitComplete()
        }
        // Make sure we have tried to update the refresh account data as well
        coVerify { mockAccountRepository.refreshAccountData() }
    }

    @Test
    fun `when logged out should navigate to Login`() = runTest {
        // Arrange
        deviceStateFlow.update { DeviceState.LoggedOut }

        // Act, Assert
        splashViewModel.uiSideEffect.test {
            assertEquals(expected = SplashUiSideEffect.NavigateToLogin, actual = awaitItem())
            awaitComplete()
        }
    }

    @Test
    fun `when revoked should navigate to Revoked`() = runTest {
        // Arrange
        deviceStateFlow.update { DeviceState.Revoked }

        // Act, Assert
        splashViewModel.uiSideEffect.test {
            assertEquals(expected = SplashUiSideEffect.NavigateToRevoked, actual = awaitItem())
            awaitComplete()
        }
    }
}
