package net.mullvad.mullvadvpn.feature.deleteaccount.impl.deleteaccountconfirmation

import app.cash.turbine.test
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import java.time.ZonedDateTime
import kotlin.test.assertEquals
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.lib.model.AccountData
import net.mullvad.mullvadvpn.lib.repository.AccountRepository
import net.mullvad.mullvadvpn.lib.repository.DeviceRepository
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class DeleteAccountConfirmationViewModelTest {

    private lateinit var mockAccountRepository: AccountRepository
    private lateinit var mockDeviceRepository: DeviceRepository

    private lateinit var viewModel: DeleteAccountConfirmationViewModel

    @BeforeEach
    fun setup() {
        mockAccountRepository = mockk()
        mockDeviceRepository = mockk(relaxed = true)

        viewModel =
            DeleteAccountConfirmationViewModel(
                accountRepository = mockAccountRepository,
                deviceRepository = mockDeviceRepository,
            )
    }

    @Test
    fun `when account expiry is in the future ui state should include the correct amount of days`() =
        runTest {
            // Arrange
            val accountData: AccountData = mockk(relaxed = true)
            every { accountData.expiryDate } returns ZonedDateTime.now().plusDays(3).plusHours(2)
            coEvery { mockAccountRepository.fetchAccountData(any(), any()) } returns accountData

            // Act, Assert
            viewModel.uiState.test { assertEquals(DaysLeftState.DaysLeft(3), awaitItem().daysLeft) }
        }

    @Test
    fun `when account expiry is in the past ui state should be no days left`() = runTest {
        // Arrange
        val accountData: AccountData = mockk(relaxed = true)
        every { accountData.expiryDate } returns ZonedDateTime.now().minusDays(4)
        coEvery { mockAccountRepository.fetchAccountData(any(), any()) } returns accountData

        // Act, Assert
        viewModel.uiState.test { assertEquals(DaysLeftState.NoDaysLeft, awaitItem().daysLeft) }
    }

    @Test
    fun `when account expiry fetching returns null out ui state should be days left error`() =
        runTest {
            // Arrange
            val accountData: AccountData = mockk(relaxed = true)
            every { accountData.expiryDate } returns ZonedDateTime.now().minusDays(4)
            coEvery { mockAccountRepository.fetchAccountData(any(), any()) } returns null

            // Act, Assert
            viewModel.uiState.test { assertEquals(DaysLeftState.Error, awaitItem().daysLeft) }
        }
}
