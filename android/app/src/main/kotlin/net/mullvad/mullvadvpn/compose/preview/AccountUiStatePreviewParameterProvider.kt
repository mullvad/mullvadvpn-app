package net.mullvad.mullvadvpn.compose.preview

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.PaymentState
import net.mullvad.mullvadvpn.lib.model.AccountNumber
import net.mullvad.mullvadvpn.lib.payment.model.PaymentProduct
import net.mullvad.mullvadvpn.lib.payment.model.PaymentStatus
import net.mullvad.mullvadvpn.lib.payment.model.ProductId
import net.mullvad.mullvadvpn.lib.payment.model.ProductPrice
import net.mullvad.mullvadvpn.viewmodel.AccountUiState
import org.joda.time.DateTime

class AccountUiStatePreviewParameterProvider : PreviewParameterProvider<AccountUiState> {
    override val values =
        sequenceOf(
            AccountUiState(
                deviceName = "Test Name",
                accountNumber = AccountNumber("1234123412341234"),
                accountExpiry = DateTime.parse("2050-12-01"),
                showSitePayment = true,
                billingPaymentState =
                    PaymentState.PaymentAvailable(
                        listOf(
                            PaymentProduct(
                                ProductId("productId"),
                                price = ProductPrice("34 SEK"),
                                status = null,
                            ),
                            PaymentProduct(
                                ProductId("productId_pending"),
                                price = ProductPrice("34 SEK"),
                                status = PaymentStatus.PENDING,
                            ),
                        )
                    ),
                showLogoutLoading = false,
                showManageAccountLoading = false,
            )
        ) + generateOtherStates()

    private fun generateOtherStates(): Sequence<AccountUiState> =
        sequenceOf(
                PaymentState.Loading,
                PaymentState.NoPayment,
                PaymentState.NoProductsFounds,
                PaymentState.Error.Billing,
            )
            .map { state ->
                AccountUiState(
                    deviceName = "Test Name",
                    accountNumber = AccountNumber("1234123412341234"),
                    accountExpiry = null,
                    showSitePayment = false,
                    billingPaymentState = state,
                    showLogoutLoading = false,
                    showManageAccountLoading = false,
                )
            }
}
