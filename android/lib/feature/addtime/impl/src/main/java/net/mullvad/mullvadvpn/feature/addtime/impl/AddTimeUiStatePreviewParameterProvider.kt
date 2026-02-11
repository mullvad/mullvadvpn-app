package net.mullvad.mullvadvpn.feature.addtime.impl

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.common.toLc
import net.mullvad.mullvadvpn.lib.payment.model.PaymentProduct
import net.mullvad.mullvadvpn.lib.payment.model.PaymentStatus
import net.mullvad.mullvadvpn.lib.payment.model.ProductId
import net.mullvad.mullvadvpn.lib.payment.model.ProductPrice

class AddTimeUiStatePreviewParameterProvider : PreviewParameterProvider<Lc<Unit, AddTimeUiState>> {
    override val values: Sequence<Lc<Unit, AddTimeUiState>> =
        sequenceOf(
            Lc.Loading(Unit),
            AddTimeUiState(
                    purchaseState = null,
                    billingPaymentState = PaymentState.Loading,
                    showSitePayment = true,
                    tunnelStateBlocked = false,
                )
                .toLc(),
            AddTimeUiState(
                    purchaseState = null,
                    billingPaymentState = PaymentState.NoPayment,
                    showSitePayment = true,
                    tunnelStateBlocked = false,
                )
                .toLc(),
        ) +
            generatePaymentStates().map { state ->
                AddTimeUiState(
                        purchaseState = null,
                        billingPaymentState = state,
                        showSitePayment = false,
                        tunnelStateBlocked = false,
                    )
                    .toLc()
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
