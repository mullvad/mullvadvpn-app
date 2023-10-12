package net.mullvad.mullvadvpn.viewmodel

import android.content.res.Resources
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import io.mockk.unmockkAll
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.model.VoucherSubmissionError
import net.mullvad.mullvadvpn.model.VoucherSubmissionResult
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionContainer
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.VoucherRedeemer
import org.junit.After
import org.junit.Before
import org.junit.Ignore
import org.junit.Rule
import org.junit.Test

class VoucherDialogViewModelTest {
    @get:Rule val testCoroutineRule = TestCoroutineRule()

    private val mockServiceConnectionManager: ServiceConnectionManager = mockk()
    private val mockVoucherRedeemer: VoucherRedeemer = mockk()
    private val mockServiceConnectionContainer: ServiceConnectionContainer = mockk()
    private val mockResources: Resources = mockk()

    private val mockVoucherSubmissionErrorResult: VoucherSubmissionResult =
        VoucherSubmissionResult.Error(VoucherSubmissionError.OtherError)

    private lateinit var viewModel: VoucherDialogViewModel

    @Before
    fun setUp() {
        mockkStatic(CACHE_EXTENSION_CLASS)
        every { mockServiceConnectionManager.connectionState.value.readyContainer() } returns
            mockServiceConnectionContainer
        every { mockServiceConnectionContainer.voucherRedeemer } returns mockVoucherRedeemer

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
    @Ignore("TODO: Fix this failing test and then enable it again.")
    fun test_submit_invalid_voucher() = runTest {
        val voucher = DUMMY_VALID_VOUCHER
        val dummyStringResource = DUMMY_STRING_RESOURCE
        // Arrange
        every { mockResources.getString(any()) } returns dummyStringResource
        coEvery { mockVoucherRedeemer.submit(voucher) } returns mockVoucherSubmissionErrorResult
        // Act, Assert
        viewModel.onRedeem(voucher)
        coVerify(exactly = 1) { mockVoucherRedeemer.submit(voucher) }
    }

    companion object {
        private const val CACHE_EXTENSION_CLASS = "net.mullvad.mullvadvpn.util.CacheExtensionsKt"
        private const val DUMMY_VALID_VOUCHER = "DUMMY_VALID_VOUCHER"
        private const val DUMMY_STRING_RESOURCE = "DUMMY_STRING_RESOURCE"
    }
}
