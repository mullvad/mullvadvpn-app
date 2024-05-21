package net.mullvad.mullvadvpn.viewmodel

import app.cash.turbine.test
import arrow.core.left
import arrow.core.right
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.unmockkAll
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlin.test.assertTrue
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.compose.state.VoucherDialogState
import net.mullvad.mullvadvpn.lib.account.VoucherRepository
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.model.RedeemVoucherError
import net.mullvad.mullvadvpn.model.RedeemVoucherSuccess
import org.joda.time.DateTime
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class VoucherDialogViewModelTest {

    private val mockVoucherSubmission: RedeemVoucherSuccess = mockk()

    private val mockVoucherRepository: VoucherRepository = mockk()

    private lateinit var viewModel: VoucherDialogViewModel

    @BeforeEach
    fun setup() {
        viewModel = VoucherDialogViewModel(voucherRepository = mockVoucherRepository)
    }

    @AfterEach
    fun tearDown() {
        unmockkAll()
    }

    @Test
    fun `ensure onRedeem invokes submit on VoucherRedeemer with same voucher code`() = runTest {
        val voucher = DUMMY_INVALID_VOUCHER

        // Arrange
        val timeAdded = 0L
        val newExpiry = DateTime()
        coEvery { mockVoucherRepository.submitVoucher(voucher) } returns
            RedeemVoucherSuccess(timeAdded, newExpiry).right()

        // Act
        assertIs<VoucherDialogState.Default>(viewModel.uiState.value.voucherState)
        viewModel.onRedeem(voucher)

        // Assert
        coVerify(exactly = 1) { mockVoucherRepository.submitVoucher(voucher) }
    }

    @Test
    fun `given invalid voucher when redeeming then show error`() = runTest {
        val voucher = DUMMY_INVALID_VOUCHER

        // Arrange
        every { mockVoucherSubmission.timeAdded } returns 0
        coEvery { mockVoucherRepository.submitVoucher(voucher) } returns
            RedeemVoucherError.InvalidVoucher.left()

        // Act, Assert
        viewModel.uiState.test {
            assertEquals(viewModel.uiState.value, awaitItem())
            viewModel.onRedeem(voucher)
            assertTrue { awaitItem().voucherState is VoucherDialogState.Verifying }
            assertTrue { awaitItem().voucherState is VoucherDialogState.Error }
        }
    }

    @Test
    fun `given valid voucher when redeeming then show success`() = runTest {
        val voucher = DUMMY_VALID_VOUCHER

        // Arrange
        every { mockVoucherSubmission.timeAdded } returns 0
        coEvery { mockVoucherRepository.submitVoucher(voucher) } returns
            RedeemVoucherSuccess(0, DateTime()).right()

        // Act, Assert
        viewModel.uiState.test {
            assertEquals(viewModel.uiState.value, awaitItem())
            viewModel.onRedeem(voucher)
            assertTrue { awaitItem().voucherState is VoucherDialogState.Verifying }
            assertTrue { awaitItem().voucherState is VoucherDialogState.Success }
        }
    }

    @Test
    fun `when voucher input is changed then clear error`() = runTest {
        val voucher = DUMMY_INVALID_VOUCHER

        // Arrange
        every { mockVoucherSubmission.timeAdded } returns 0
        coEvery { mockVoucherRepository.submitVoucher(voucher) } returns
            RedeemVoucherError.VoucherAlreadyUsed.left()

        // Act, Assert
        viewModel.uiState.test {
            assertEquals(viewModel.uiState.value, awaitItem())
            viewModel.onRedeem(voucher)
            assertTrue { awaitItem().voucherState is VoucherDialogState.Verifying }
            assertTrue { awaitItem().voucherState is VoucherDialogState.Error }
            viewModel.onVoucherInputChange(DUMMY_VALID_VOUCHER)
            assertTrue { awaitItem().voucherState is VoucherDialogState.Default }
        }
    }

    companion object {
        private const val DUMMY_VALID_VOUCHER = "dummy_valid_voucher"
        private const val DUMMY_INVALID_VOUCHER = "dummy_invalid_voucher"
    }
}
