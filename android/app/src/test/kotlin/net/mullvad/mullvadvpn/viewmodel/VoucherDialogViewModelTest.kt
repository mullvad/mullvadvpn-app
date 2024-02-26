package net.mullvad.mullvadvpn.viewmodel

import android.content.res.Resources
import app.cash.turbine.test
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.unmockkAll
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlin.test.assertTrue
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.compose.state.VoucherDialogState
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.model.VoucherSubmission
import net.mullvad.mullvadvpn.model.VoucherSubmissionError
import net.mullvad.mullvadvpn.model.VoucherSubmissionResult
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionContainer
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionState
import net.mullvad.mullvadvpn.ui.serviceconnection.VoucherRedeemer
import net.mullvad.mullvadvpn.ui.serviceconnection.voucherRedeemer
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class VoucherDialogViewModelTest {

    private val mockServiceConnectionManager: ServiceConnectionManager = mockk()
    private val mockServiceConnectionContainer: ServiceConnectionContainer = mockk()
    private val mockVoucherSubmission: VoucherSubmission = mockk()
    private val serviceConnectionState =
        MutableStateFlow<ServiceConnectionState>(ServiceConnectionState.Disconnected)

    private val mockVoucherRedeemer: VoucherRedeemer = mockk()
    private val mockResources: Resources = mockk()

    private lateinit var viewModel: VoucherDialogViewModel

    @BeforeEach
    fun setup() {
        every { mockServiceConnectionManager.connectionState } returns serviceConnectionState

        viewModel =
            VoucherDialogViewModel(
                serviceConnectionManager = mockServiceConnectionManager,
                resources = mockResources
            )
    }

    @AfterEach
    fun tearDown() {
        unmockkAll()
    }

    @Test
    fun `ensure onRedeem invokes submit on VoucherRedeemer with same voucher code`() = runTest {
        val voucher = DUMMY_INVALID_VOUCHER

        // Arrange
        every { mockServiceConnectionManager.voucherRedeemer() } returns mockVoucherRedeemer
        every { mockVoucherSubmission.timeAdded } returns 0
        coEvery { mockVoucherRedeemer.submit(voucher) } returns
            VoucherSubmissionResult.Ok(mockVoucherSubmission)

        // Act
        assertIs<VoucherDialogState.Default>(viewModel.uiState.value.voucherState)
        viewModel.onRedeem(voucher)

        // Assert
        coVerify(exactly = 1) { mockVoucherRedeemer.submit(voucher) }
    }

    @Test
    fun `given invalid voucher when redeeming then show error`() = runTest {
        val voucher = DUMMY_INVALID_VOUCHER
        val dummyStringResource = DUMMY_STRING_RESOURCE

        // Arrange
        every { mockServiceConnectionManager.voucherRedeemer() } returns mockVoucherRedeemer
        every { mockResources.getString(any()) } returns dummyStringResource
        every { mockVoucherSubmission.timeAdded } returns 0
        coEvery { mockVoucherRedeemer.submit(voucher) } returns
            VoucherSubmissionResult.Error(VoucherSubmissionError.OtherError)

        // Act, Assert
        viewModel.uiState.test {
            assertEquals(viewModel.uiState.value, awaitItem())
            serviceConnectionState.value =
                ServiceConnectionState.ConnectedReady(mockServiceConnectionContainer)
            viewModel.onRedeem(voucher)
            assertTrue { awaitItem().voucherState is VoucherDialogState.Verifying }
            assertTrue { awaitItem().voucherState is VoucherDialogState.Error }
        }
    }

    @Test
    fun `given valid voucher when redeeming then show success`() = runTest {
        val voucher = DUMMY_VALID_VOUCHER
        val dummyStringResource = DUMMY_STRING_RESOURCE

        // Arrange
        every { mockServiceConnectionManager.voucherRedeemer() } returns mockVoucherRedeemer
        every { mockResources.getString(any()) } returns dummyStringResource
        every { mockVoucherSubmission.timeAdded } returns 0
        coEvery { mockVoucherRedeemer.submit(voucher) } returns
            VoucherSubmissionResult.Ok(VoucherSubmission(0, DUMMY_STRING_RESOURCE))

        // Act, Assert
        viewModel.uiState.test {
            assertEquals(viewModel.uiState.value, awaitItem())
            serviceConnectionState.value =
                ServiceConnectionState.ConnectedReady(mockServiceConnectionContainer)
            viewModel.onRedeem(voucher)
            assertTrue { awaitItem().voucherState is VoucherDialogState.Verifying }
            assertTrue { awaitItem().voucherState is VoucherDialogState.Success }
        }
    }

    @Test
    fun `when voucher input is changed then clear error`() = runTest {
        val voucher = DUMMY_INVALID_VOUCHER
        val dummyStringResource = DUMMY_STRING_RESOURCE

        // Arrange
        every { mockServiceConnectionManager.voucherRedeemer() } returns mockVoucherRedeemer
        every { mockResources.getString(any()) } returns dummyStringResource
        every { mockVoucherSubmission.timeAdded } returns 0
        coEvery { mockVoucherRedeemer.submit(voucher) } returns
            VoucherSubmissionResult.Error(VoucherSubmissionError.OtherError)

        // Act, Assert
        viewModel.uiState.test {
            assertEquals(viewModel.uiState.value, awaitItem())
            serviceConnectionState.value =
                ServiceConnectionState.ConnectedReady(mockServiceConnectionContainer)
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
        private const val DUMMY_STRING_RESOURCE = "dummy_string_resource"
    }
}
