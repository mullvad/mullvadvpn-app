package net.mullvad.mullvadvpn.compose.preview

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.AddMoreTimeUiState
import net.mullvad.mullvadvpn.compose.state.PaymentState
import net.mullvad.mullvadvpn.lib.payment.model.PaymentProduct
import net.mullvad.mullvadvpn.lib.payment.model.PaymentStatus
import net.mullvad.mullvadvpn.lib.payment.model.ProductId
import net.mullvad.mullvadvpn.lib.payment.model.ProductPrice
import net.mullvad.mullvadvpn.util.Lce

class AddMoreTimeUiStatePreviewParameterProvider :
    PreviewParameterProvider<Lce<Unit, AddMoreTimeUiState, Unit>> {
    override val values: Sequence<Lce<Unit, AddMoreTimeUiState, Unit>> =
        sequenceOf(
            Lce.Loading(Unit),
            Lce.Content(
                AddMoreTimeUiState(
                    billingPaymentState = null,
                    showSitePayment = true,
                    showManageAccountLoading = false,
                )
            ),
            Lce.Content(
                AddMoreTimeUiState(
                    billingPaymentState = null,
                    showSitePayment = true,
                    showManageAccountLoading = true,
                )
            ),
        ) +
            generatePaymentStates().map { state ->
                Lce.Content(
                    AddMoreTimeUiState(
                        billingPaymentState = state,
                        showSitePayment = false,
                        showManageAccountLoading = true,
                    )
                )
            }

    private fun generatePaymentStates(): Sequence<PaymentState> =
        sequenceOf(
            PaymentState.Loading,
            PaymentState.NoPayment,
            PaymentState.NoProductsFounds,
            PaymentState.PaymentAvailable(
                products =
                    listOf(
                        PaymentProduct(
                            productId = ProductId("one_month"),
                            price = ProductPrice("$10"),
                            status = null,
                        ),
                        PaymentProduct(
                            productId = ProductId("three_months"),
                            price = ProductPrice("$30"),
                            status = null,
                        ),
                    )
            ),
            PaymentState.PaymentAvailable(
                products =
                    listOf(
                        PaymentProduct(
                            productId = ProductId("one_month"),
                            price = ProductPrice("$10"),
                            status = PaymentStatus.PENDING,
                        ),
                        PaymentProduct(
                            productId = ProductId("three_months"),
                            price = ProductPrice("$30"),
                            status = null,
                        ),
                    )
            ),
            PaymentState.Error.Billing,
        )
}
