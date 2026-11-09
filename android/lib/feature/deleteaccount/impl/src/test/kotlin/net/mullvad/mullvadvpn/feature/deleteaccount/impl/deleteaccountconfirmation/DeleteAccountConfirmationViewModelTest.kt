package net.mullvad.mullvadvpn.feature.deleteaccount.impl.deleteaccountconfirmation

import app.cash.turbine.test
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import java.time.ZonedDateTime
import kotlin.test.assertEquals
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.lib.model.AccountData
import net.mullvad.mullvadvpn.lib.repository.AccountRepository
import net.mullvad.mullvadvpn.lib.repository.DeviceRepository
import org.junit.jupiter.api.Test

class DeleteAccountConfirmationViewModelTest {

    private val mockAccountRepository: AccountRepository = mockk(relaxed = true)
    private val mockDeviceRepository: DeviceRepository = mockk(relaxed = true)

    private val viewModel =
        DeleteAccountConfirmationViewModel(
            accountRepository = mockAccountRepository,
            deviceRepository = mockDeviceRepository,
        )

    @Test
    fun `when account expiry is in the future ui state should include the correct amount of days`() =
        runTest {
            // Arrange
            val accountData: AccountData = mockk(relaxed = true)
            every { accountData.expiryDate } returns ZonedDateTime.now().plusDays(4)
            coEvery { mockAccountRepository.fetchAccountData(any(), any()) } returns accountData

            // Act, Assert
            viewModel.uiState.test {
                assertEquals(DaysLeftState.Loading, awaitItem().daysLeft)
                assertEquals(DaysLeftState.DaysLeft(3), awaitItem().daysLeft)
            }
        }

    @Test
    fun `when account expiry is in the past ui state should be no days left`() = runTest {
        // Arrange
        val accountData: AccountData = mockk(relaxed = true)
        every { accountData.expiryDate } returns ZonedDateTime.now().minusDays(4)
        coEvery { mockAccountRepository.fetchAccountData(any(), any()) } returns accountData

        // Act, Assert
        viewModel.uiState.test {
            assertEquals(DaysLeftState.Loading, awaitItem().daysLeft)
            assertEquals(DaysLeftState.NoDaysLeft, awaitItem().daysLeft)
        }
    }

    @Test
    fun `when account expiry fetching returns null out ui state should be days left error`() =
        runTest {
            // Arrange
            val accountData: AccountData = mockk(relaxed = true)
            every { accountData.expiryDate } returns ZonedDateTime.now().minusDays(4)
            coEvery { mockAccountRepository.fetchAccountData(any(), any()) } returns null

            // Act, Assert
            viewModel.uiState.test {
                assertEquals(DaysLeftState.Loading, awaitItem().daysLeft)
                assertEquals(DaysLeftState.Error, awaitItem().daysLeft)
            }
        }
}
