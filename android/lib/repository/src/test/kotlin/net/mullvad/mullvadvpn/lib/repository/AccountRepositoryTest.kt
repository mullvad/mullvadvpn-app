package net.mullvad.mullvadvpn.lib.repository

import app.cash.turbine.test
import arrow.core.right
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlin.test.assertEquals
import kotlinx.coroutines.MainScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.lib.grpc.ManagementService
import net.mullvad.mullvadvpn.lib.model.AccountData
import net.mullvad.mullvadvpn.lib.model.AccountNumber
import net.mullvad.mullvadvpn.lib.model.DeviceState
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.assertNull
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class AccountRepositoryTest {

    private val mockManagementService: ManagementService = mockk()
    private val mockDeviceRepository: DeviceRepository = mockk()

    private val mockDeviceStateFlow = MutableStateFlow<DeviceState>(DeviceState.LoggedOut)

    private lateinit var accountRepository: AccountRepository

    @BeforeEach
    fun setup() {
        every { mockDeviceRepository.deviceState } returns mockDeviceStateFlow
        every { mockManagementService.deviceState } returns mockDeviceStateFlow

        accountRepository =
            AccountRepository(
                managementService = mockManagementService,
                deviceRepository = mockDeviceRepository,
                scope = MainScope(),
            )
    }

    @Test
    fun `given force is true should always call managementService getAccountData`() = runTest {
        // Arrange
        val accountData: AccountData = mockk()
        val accountNumber = AccountNumber("1234567890")
        every { accountData.accountNumber } returns accountNumber
        coEvery { mockManagementService.getAccountData(accountNumber) } returns accountData.right()

        // Act
        mockDeviceStateFlow.emit(DeviceState.LoggedIn(accountNumber, mockk(relaxed = true)))
        accountRepository.refreshAccountData(ignoreTimeout = true)

        // Assert
        coVerify { mockManagementService.getAccountData(accountNumber) }
    }

    @Test
    fun `given last latestAccountDataFetch null should always call managementService getAccountData`() =
        runTest {
            // Arrange
            val accountData: AccountData = mockk()
            val accountNumber = AccountNumber("1234567890")
            every { accountData.accountNumber } returns accountNumber
            coEvery { mockManagementService.getAccountData(accountNumber) } returns
                accountData.right()

            // Act
            mockDeviceStateFlow.emit(DeviceState.LoggedIn(accountNumber, mockk(relaxed = true)))
            accountRepository.refreshAccountData(ignoreTimeout = false)

            // Assert
            coVerify { mockManagementService.getAccountData(accountNumber) }
        }

    @Test
    fun `when an account is successfully deleted should clear account data cache`() = runTest {
        // Arrange
        val accountNumber = AccountNumber("1234567890")
        val accountData: AccountData = mockk()
        every { accountData.accountNumber } returns accountNumber
        coEvery { mockManagementService.getAccountData(accountNumber) } returns accountData.right()
        coEvery { mockManagementService.deleteAccount() } returns Unit.right()
        mockDeviceStateFlow.value =
            DeviceState.LoggedIn(
                accountNumber = accountNumber,
                mockk(),
            )

        // Act, Assert
        accountRepository.refreshAccountData()
        accountRepository.accountData.test {
            // Check that the current account data is correct
            assertEquals(accountData, awaitItem())
            // Delete account
            accountRepository.deleteAccount()
            // Check that account data is null
            assertNull(awaitItem())
        }
    }

    @Test
    fun `when deleteAccount is called should call ManagementService delete account`() = runTest {
        // Arrange
        coEvery { mockManagementService.deleteAccount() } returns Unit.right()

        // Acy
        accountRepository.deleteAccount()

        // Assert
        coVerify { mockManagementService.deleteAccount() }
    }

    @Test
    fun `when an account is successfully logged out should clear account data cache`() = runTest {
        // Arrange
        val accountNumber = AccountNumber("1234567890")
        val accountData: AccountData = mockk()
        every { accountData.accountNumber } returns accountNumber
        coEvery { mockManagementService.getAccountData(accountNumber) } returns accountData.right()
        coEvery { mockManagementService.logoutAccount() } returns Unit.right()
        mockDeviceStateFlow.value =
            DeviceState.LoggedIn(
                accountNumber = accountNumber,
                mockk(),
            )

        // Act, Assert
        accountRepository.refreshAccountData()
        accountRepository.accountData.test {
            // Check that the current account data is correct
            assertEquals(accountData, awaitItem())
            // Logout account
            accountRepository.logout()
            // Check that account data is null
            assertNull(awaitItem())
        }
    }
}
