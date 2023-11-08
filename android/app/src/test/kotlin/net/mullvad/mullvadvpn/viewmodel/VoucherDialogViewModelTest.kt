package net.mullvad.mullvadvpn.viewmodel

import android.content.res.Resources
import io.mockk.MockKAnnotations
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import io.mockk.unmockkAll
import kotlin.test.assertIs
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.compose.state.VoucherDialogState
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.model.VoucherSubmissionError
import net.mullvad.mullvadvpn.model.VoucherSubmissionResult
import net.mullvad.mullvadvpn.ui.serviceconnection.ConnectionProxy
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionContainer
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionState
import net.mullvad.mullvadvpn.ui.serviceconnection.VoucherRedeemer
import net.mullvad.mullvadvpn.ui.serviceconnection.voucherRedeemer
import org.junit.After
import org.junit.Before
import org.junit.Rule
import org.junit.Test

class VoucherDialogViewModelTest {
    @get:Rule val testCoroutineRule = TestCoroutineRule()

    private val mockServiceConnectionManager: ServiceConnectionManager = mockk()
    private val mockServiceConnectionContainer: ServiceConnectionContainer = mockk()
    private val mockConnectionProxy: ConnectionProxy = mockk()
    private val serviceConnectionState =
        MutableStateFlow<ServiceConnectionState>(ServiceConnectionState.Disconnected)

    private val mockVoucherRedeemer: VoucherRedeemer = mockk()
    private val mockResources: Resources = mockk()

    private val mockVoucherSubmissionErrorResult: VoucherSubmissionResult =
        VoucherSubmissionResult.Error(VoucherSubmissionError.OtherError)

    private lateinit var viewModel: VoucherDialogViewModel

    @Before
    fun setUp() {
        MockKAnnotations.init(this)
        mockkStatic(CACHE_EXTENSION_CLASS)
        every { mockServiceConnectionManager.connectionState } returns serviceConnectionState
        every { mockServiceConnectionContainer.voucherRedeemer } returns mockVoucherRedeemer
        every { mockServiceConnectionContainer.connectionProxy } returns mockConnectionProxy

        viewModel =
            VoucherDialogViewModel(
                serviceConnectionManager = mockServiceConnectionManager,
                resources = mockResources
            )
    }

    @After
    fun tearDown() {
        unmockkAll()
    }

    @Test
    fun test_submit_voucher() = runTest {
        val voucher = DUMMY_INVALID_VOUCHER
        val dummyStringResource = DUMMY_STRING_RESOURCE
        // Arrange
        serviceConnectionState.value =
            ServiceConnectionState.ConnectedReady(mockServiceConnectionContainer)
        every { mockServiceConnectionManager.voucherRedeemer() } returns mockVoucherRedeemer
        every { mockResources.getString(any()) } returns dummyStringResource
        coEvery { mockVoucherRedeemer.submit(voucher) } returns mockVoucherSubmissionErrorResult
        // Act, Assert
        assertIs<VoucherDialogState.Default>(viewModel.uiState.value.voucherViewModelState)
        viewModel.onRedeem(voucher)
        coVerify(exactly = 1) { mockVoucherRedeemer.submit(voucher) }
    }

    companion object {
        private const val CACHE_EXTENSION_CLASS = "net.mullvad.mullvadvpn.util.CacheExtensionsKt"
        private const val DUMMY_INVALID_VOUCHER = "dummy_invalid_voucher"
        private const val DUMMY_STRING_RESOURCE = "dummy_string_resource"
    }
}
