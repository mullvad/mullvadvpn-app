package net.mullvad.mullvadvpn.viewmodel

import app.cash.turbine.test
import io.mockk.coEvery
import io.mockk.mockk
import io.mockk.unmockkAll
import kotlin.test.assertEquals
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.compose.dialog.payment.PaymentDialogData
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.lib.payment.model.PurchaseResult
import net.mullvad.mullvadvpn.usecase.PaymentUseCase
import net.mullvad.mullvadvpn.util.toPaymentDialogData
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(TestCoroutineRule::class)
class PaymentViewModelTest {

    private val mockPaymentUseCase: PaymentUseCase = mockk(relaxed = true)

    private val purchaseResult = MutableStateFlow<PurchaseResult?>(null)

    private lateinit var viewModel: PaymentViewModel

    @BeforeEach
    fun setup() {
        coEvery { mockPaymentUseCase.purchaseResult } returns purchaseResult

        viewModel = PaymentViewModel(paymentUseCase = mockPaymentUseCase)
    }

    @AfterEach
    fun tearDown() {
        unmockkAll()
    }

    @Test
    fun `given PaymentUseCase purchaseResult emits cancelled uiSideEffect should emit PaymentCancelled`() =
        runTest {
            // Arrange
            val result = PurchaseResult.Completed.Cancelled
            purchaseResult.value = result

            // Act, Assert
            viewModel.uiState.test {
                assertEquals(PaymentDialogData(), awaitItem().paymentDialogData)
                purchaseResult.value = result
            }

            viewModel.uiSideEffect.test {
                assertEquals(PaymentUiSideEffect.PaymentCancelled, awaitItem())
            }
        }

    @Test
    fun `given purchaseResult emits Success uiState should return success dialog data`() = runTest {
        // Arrange
        val result = PurchaseResult.Completed.Success

        // Act, Assert
        viewModel.uiState.test {
            awaitItem()
            purchaseResult.value = result
            assertEquals(result.toPaymentDialogData(), awaitItem().paymentDialogData)
        }
    }
}
