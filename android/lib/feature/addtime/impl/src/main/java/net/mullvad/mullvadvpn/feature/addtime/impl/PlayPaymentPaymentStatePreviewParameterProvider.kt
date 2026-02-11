package net.mullvad.mullvadvpn.feature.addtime.impl

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.lib.payment.model.PaymentProduct
import net.mullvad.mullvadvpn.lib.payment.model.PaymentStatus
import net.mullvad.mullvadvpn.lib.payment.model.ProductId
import net.mullvad.mullvadvpn.lib.payment.model.ProductPrice

class PlayPaymentPaymentStatePreviewParameterProvider : PreviewParameterProvider<PaymentState> {
    override val values: Sequence<PaymentState> =
        sequenceOf(PaymentState.Loading, PaymentState.Error.Generic, PaymentState.Error.Billing) +
            sequenceOf(
                // Products available
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
                // Product pending
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
                // Product verification in progress
                PaymentState.PaymentAvailable(
                    products =
                        listOf(
                            PaymentProduct(
                                productId = ProductId("one_month"),
                                price = ProductPrice("$10"),
                                status = PaymentStatus.VERIFICATION_IN_PROGRESS,
                            ),
                            PaymentProduct(
                                productId = ProductId("three_months"),
                                price = ProductPrice("$30"),
                                status = null,
                            ),
                        )
                ),
            )
}
