package net.mullvad.mullvadvpn.viewmodel

import app.cash.turbine.test
import arrow.core.left
import arrow.core.right
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.unmockkAll
import java.time.ZonedDateTime
import kotlin.test.assertIs
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.compose.state.VoucherDialogState
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.lib.model.RedeemVoucherError
import net.mullvad.mullvadvpn.lib.model.RedeemVoucherSuccess
import net.mullvad.mullvadvpn.lib.model.VoucherCode
import net.mullvad.mullvadvpn.lib.repository.VoucherRepository
import net.mullvad.mullvadvpn.usecase.InternetAvailableUseCase
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class VoucherDialogViewModelTest {

    private val mockVoucherSubmission: RedeemVoucherSuccess = mockk()

    private val mockVoucherRepository: VoucherRepository = mockk()

    private val mockInternetAvailableUseCase: InternetAvailableUseCase = mockk()

    private lateinit var viewModel: VoucherDialogViewModel

    @BeforeEach
    fun setup() {
        viewModel =
            VoucherDialogViewModel(
                voucherRepository = mockVoucherRepository,
                internetAvailableUseCase = mockInternetAvailableUseCase,
            )
    }

    @AfterEach
    fun tearDown() {
        unmockkAll()
    }

    @Test
    fun `ensure onRedeem invokes submit on VoucherRedeemer with same voucher code`() = runTest {
        val voucher = DUMMY_INVALID_VOUCHER
        val parsedVoucher = VoucherCode.fromString(voucher).getOrNull()!!

        // Arrange
        val timeAdded = 0L
        val newExpiry = ZonedDateTime.now()
        coEvery { mockVoucherRepository.submitVoucher(parsedVoucher) } returns
            RedeemVoucherSuccess(timeAdded, newExpiry).right()

        // Act
        assertIs<VoucherDialogState.Default>(viewModel.uiState.value.voucherState)
        viewModel.onRedeem(voucher)

        // Assert
        coVerify(exactly = 1) { mockVoucherRepository.submitVoucher(parsedVoucher) }
    }

    @Test
    fun `given invalid voucher when redeeming then show error`() = runTest {
        val voucher = DUMMY_INVALID_VOUCHER

        // Arrange
        every { mockVoucherSubmission.timeAdded } returns 0
        coEvery {
            mockVoucherRepository.submitVoucher(VoucherCode.fromString(voucher).getOrNull()!!)
        } returns RedeemVoucherError.InvalidVoucher.left()

        // Act, Assert
        viewModel.uiState.test {
            assertIs<VoucherDialogState.Default>(awaitItem().voucherState)
            viewModel.onRedeem(voucher)
            assertIs<VoucherDialogState.Verifying>(awaitItem().voucherState)
            val errorState = awaitItem().voucherState
            assertIs<VoucherDialogState.Error.DaemonError>(errorState)
            assertIs<RedeemVoucherError.InvalidVoucher>(errorState.error)
        }
    }

    @Test
    fun `given api unreachable error and internet is available should show api unreachable error`() =
        runTest {
            val voucher = DUMMY_INVALID_VOUCHER

            // Arrange
            every { mockVoucherSubmission.timeAdded } returns 0
            coEvery {
                mockVoucherRepository.submitVoucher(VoucherCode.fromString(voucher).getOrNull()!!)
            } returns RedeemVoucherError.ApiUnreachable.left()
            every { mockInternetAvailableUseCase() } returns true

            // Act, Assert
            viewModel.uiState.test {
                assertIs<VoucherDialogState.Default>(awaitItem().voucherState)
                viewModel.onRedeem(voucher)
                assertIs<VoucherDialogState.Verifying>(awaitItem().voucherState)
                val error = awaitItem()
                assertIs<VoucherDialogState.Error.DaemonError>(error.voucherState)
                assertIs<RedeemVoucherError.ApiUnreachable>(error.voucherState.error)
            }
        }

    @Test
    fun `given api unreachable error and internet is not available should show no internet error`() =
        runTest {
            val voucher = DUMMY_INVALID_VOUCHER

            // Arrange
            every { mockVoucherSubmission.timeAdded } returns 0
            coEvery {
                mockVoucherRepository.submitVoucher(VoucherCode.fromString(voucher).getOrNull()!!)
            } returns RedeemVoucherError.ApiUnreachable.left()
            every { mockInternetAvailableUseCase() } returns false

            // Act, Assert
            viewModel.uiState.test {
                assertIs<VoucherDialogState.Default>(awaitItem().voucherState)
                viewModel.onRedeem(voucher)
                assertIs<VoucherDialogState.Verifying>(awaitItem().voucherState)
                val error = awaitItem()
                assertIs<VoucherDialogState.Error.NoInternet>(error.voucherState)
            }
        }

    @Test
    fun `given valid voucher when redeeming then show success`() = runTest {
        val voucher = DUMMY_VALID_VOUCHER

        // Arrange
        every { mockVoucherSubmission.timeAdded } returns 0
        coEvery {
            mockVoucherRepository.submitVoucher(VoucherCode.fromString(voucher).getOrNull()!!)
        } returns RedeemVoucherSuccess(0, ZonedDateTime.now()).right()

        // Act, Assert
        viewModel.uiState.test {
            assertIs<VoucherDialogState.Default>(awaitItem().voucherState)
            viewModel.onRedeem(voucher)
            assertIs<VoucherDialogState.Verifying>(awaitItem().voucherState)
            assertIs<VoucherDialogState.Success>(awaitItem().voucherState)
        }
    }

    @Test
    fun `when voucher input is changed then clear error`() = runTest {
        val voucher = DUMMY_INVALID_VOUCHER

        // Arrange
        every { mockVoucherSubmission.timeAdded } returns 0
        coEvery {
            mockVoucherRepository.submitVoucher(VoucherCode.fromString(voucher).getOrNull()!!)
        } returns RedeemVoucherError.VoucherAlreadyUsed.left()

        // Act, Assert
        viewModel.uiState.test {
            assertIs<VoucherDialogState.Default>(awaitItem().voucherState)
            viewModel.onRedeem(voucher)
            assertIs<VoucherDialogState.Verifying>(awaitItem().voucherState)
            assertIs<VoucherDialogState.Error>(awaitItem().voucherState)
            viewModel.onVoucherInputChange(DUMMY_VALID_VOUCHER)
            assertIs<VoucherDialogState.Default>(awaitItem().voucherState)
        }
    }

    companion object {
        private const val DUMMY_VALID_VOUCHER = "dummy_valid_voucher"
        private const val DUMMY_INVALID_VOUCHER = "dummy_invalid_voucher"
    }
}
