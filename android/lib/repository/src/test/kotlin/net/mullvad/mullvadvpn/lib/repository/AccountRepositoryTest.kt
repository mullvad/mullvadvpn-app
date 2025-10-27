package net.mullvad.mullvadvpn.lib.repository

import arrow.core.right
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.MainScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.lib.daemon.grpc.ManagementService
import net.mullvad.mullvadvpn.lib.model.AccountData
import net.mullvad.mullvadvpn.lib.model.AccountNumber
import net.mullvad.mullvadvpn.lib.model.DeviceState
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@OptIn(ExperimentalCoroutinesApi::class)
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
}
